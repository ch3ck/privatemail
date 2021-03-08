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
    organization = "nyah.dev"
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
    enable = true
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