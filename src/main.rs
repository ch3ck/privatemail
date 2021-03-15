//! Copyright 2021 Nyah Check crate.
//!
//! privatemail - AWS Lambda SES Forwarder - v1.0.0
//! This service is for forwarding reaw emails from S3 to SES.
//! Configures S3 bucket with required prefixes to store raw emails and mapping for email addresses
//! needed to forwared the emails.
//!
//! EmailConfig:
//! from_email: Forwarded emails will be received from this SES verified email address.
//!             To match all email addresses on a domain, use a key without the name part of the email(`example.com`)
//! to_email: Recipient email address
//!             Example: jon@doe.example
//! subject_prefix: Forwarded emails subject will contain this prefix.
//! email_bucket: S3 bucket to store raw SES emails.
//! email_key_prefix: S3 key prefix where SES stores emails.
#![allow(clippy::style)]
mod lib;
use lib::PrivatEmail_Handler;
use lambda_runtime::{handler_fn, Error};


#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(PrivatEmail_Handler);
    lambda_runtime::run(PrivatEmail_Handler).await?;
    Ok(())
}
