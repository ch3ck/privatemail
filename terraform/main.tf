provider "aws" {
  profile = "default"
  region  = "us-east-1"
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
  bucket = var.bucket_name
  acl    = "private"

  tags = {
    Name        = var.bucket_name
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
      "ses:SendRawEmail"
    ]

    resources = [
      "*"
    ]
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
  name            = "ses-email-forward-sns-topic"
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
