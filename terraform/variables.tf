variable "bucket" {
  description = "Value of the S3 bucket name"
  type        = string
  default     = "nyah-ses-emails"
}

variable "function_name" {
  default     = "ses-forward-emails-lambda-nyah"
  description = "Value of Lambda function name"
  type        = string
}

variable "topic" {
  default     = "ses-email-forward-sns-topic-nyah"
  type        = string
  description = "SES email forwarding sns topic"
}

variable "from_email" {
  default     = "hello@nyah.dev"
  description = "Original email from which the email was sent to."
}

variable "to_email" {
  default     = "nyah@hey.com"
  description = "AWS Verified email for forwarding your emails"
}

variable "region" {
  default     = "us-east-1"
  description = "AWS region for deployment"
}


variable "organization" {
  default     = "nyahdev"
  description = "Terraform cloud organization name"
}

variable "workspace" {
  default     = "nyah-dot-dev-workspace"
  description = "Terraform cloud workspace name"
}

variable "rule_set_name" {
  default     = "ses-forward-rule-set-nyah"
  description = "ses forward rule set name"
}

variable "rule_name" {
  default     = "ses-forward-rule-nyah"
  description = "ses forward rule name"
}

variable "domain_name" {
  default     = "nyah.dev"
  description = "verified AWS domain"
}
