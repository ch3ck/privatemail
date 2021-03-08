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
    organization = var.organization
    workspaces {
      name = var.workspace
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

resource "aws_lambda_function" "ses-email-forward-lambda" {
  filename      = "example/lambda.zip"
  function_name = var.function_name
  role          = aws_iam_role.ses-email-role
  handler       = "privatemail_handler"

  source_code_hash = filebase64sha256("example/lambda.zip")
  runtime          = "provided"

  environment {
    variables = {
      RUST_BACKTRACE = 1
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
