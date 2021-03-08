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
