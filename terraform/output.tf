output "lambda_arn" {
  description = "Lambda Function arn"
  value       = aws_lambda_function.ses-email-forward-lambda.arn
}

output "topic_arn" {
  description = "SNS Topic Arn"
  value       = aws_sns_topic.ses-email-topic.arn
}

output "bucket_acl_grant" {
  description = "S3 Bucket ACL Grant"
  value       = aws_s3_bucket.ses-bucket.grant
}