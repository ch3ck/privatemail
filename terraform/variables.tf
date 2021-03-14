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
  default = "hello@nyah.dev"
}

variable "to_email" {
  default = "nyah@hey.com"
}

variable "region" {
  default = "us-east-1"
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
  default = "ses-forward-rule-set-nyah"
}

variable "rule_name" {
  default = "ses-forward-rule-nyah"
}

variable "domain_name" {
  default = "nyah.dev"
}
