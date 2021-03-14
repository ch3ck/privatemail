provider "aws" {
  profile = "default"
  region  = var.region
}


terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 3.27"
    }
  }
  backend "remote" {
    organization = "nyahdev"
    workspaces {
      name = "nyah-dot-dev-workspace"
    }
  }
}

resource "aws_s3_bucket" "ses-bucket" {
  bucket = var.bucket
  acl    = "private"

  tags = {
    Name        = var.bucket
    Environment = "personal"
  }

  versioning {
    enabled = true
  }
}


data "aws_iam_policy_document" "ses_email_forward_policy_document" {
  statement {
    sid = "1"

    actions = [
      "logs:CreateLogGroup",
      "logs:CreateLogStream",
      "logs:PutLogEvents",
    ]

    resources = [
      "arn:aws:logs:*:*:*",
    ]
  }

  statement {
    sid = "2"

    actions = [
      "ses:SendEmail",
      "ses:SendRawEmail",
    ]

    resources = [
      "*"
    ]
  }

  statement {
    sid = "3"

    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_policy" "ses-email-policy" {
  name   = "ses-forward-email-policy"
  path   = "/"
  policy = data.aws_iam_policy_document.ses_email_forward_policy_document.json
}


resource "aws_iam_role" "ses-email-role" {
  name                = "ses-email-forward-lambda-invoke-role"
  assume_role_policy  = data.aws_iam_policy_document.ses_email_forward_policy_document.json
  managed_policy_arns = [aws_iam_policy.ses-email-policy.arn]
}

resource "aws_sns_topic" "ses-email-topic" {
  name            = var.topic
  delivery_policy = <<EOF
{
  "http": {
    "defaultHealthyRetryPolicy": {
      "minDelayTarget": 20,
      "maxDelayTarget": 20,
      "numRetries": 3,
      "numMaxDelayRetries": 0,
      "numNoDelayRetries": 0,
      "numMinDelayRetries": 0,
      "backoffFunction": "linear"
    },
    "disableSubscriptionOverrides": false
  }
}
EOF
}

resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = 14
}

resource "aws_iam_policy" "cloudwatch_lambda_logs" {
  name        = "cloudwatch_lambda_logs"
  path        = "/"
  description = "IAM policy for logging from a lambda"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*",
      "Effect": "Allow"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "lambda_logs_policy" {
  role       = aws_iam_role.ses-email-role.name
  policy_arn = aws_iam_policy.cloudwatch_lambda_logs.arn
}

resource "aws_lambda_function" "ses-email-forward-lambda" {
  filename      = "lambda.zip"
  function_name = var.function_name
  role          = aws_iam_role.ses-email-role.arn
  handler       = "privatemail_handler"

  source_code_hash = filebase64sha256("lambda.zip")
  runtime          = "provided"

  # cloudwatch logging
  depends_on = [
    aws_iam_role_policy_attachment.lambda_logs_policy,
    aws_cloudwatch_log_group.lambda_log_group,
  ]

  environment {
    variables = {
      RUST_BACKTRACE = 1,
      FROM_EMAIL     = var.from_email,
      TO_EMAIL       = var.to_email
    }
  }
}

resource "aws_lambda_permission" "allow_sns_trigger" {
  statement_id  = "AllowExecutionFromSNS"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.ses-email-forward-lambda.function_name
  principal     = "sns.amazonaws.com"
  source_arn    = aws_sns_topic.ses-email-topic.arn
}

resource "aws_sns_topic_subscription" "lambda_subscription" {
  topic_arn = aws_sns_topic.ses-email-topic.arn
  protocol  = "lambda"
  endpoint  = aws_lambda_function.ses-email-forward-lambda.arn
}
