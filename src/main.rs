//! Copyright 2021 Nyah Check crate.
//!
//! privatemail - AWS Lambda SES Forwarder - v1.0.0
//! This service is for forwarding reaw emails from S3 to SES.
//! Configures S3 bucket with required prefixes to store raw emails and mapping for email addresses
//! needed to forwared the emails.
#![allow(clippy::style)]
mod lib;
use lib::PrivatEmail_Handler;
use lambda_runtime::{handler_fn, Error};


#[tokio::main]
async fn main() -> Result<(), Error> {
    let privatemail_handler = handler_fn(PrivatEmail_Handler);
    lambda_runtime::run(privatemail_handler).await?;
    Ok(())
}
