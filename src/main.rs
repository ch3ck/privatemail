//! Copyright 2021 Nyah Check crate.
//!
//! privatemail - AWS Lambda SES Forwarder - v1.0.0
//! This service is for forwarding reaw emails from S3 to SES.
//! Configures S3 bucket with required prefixes to store raw emails and mapping for email addresses
//! needed to forwared the emails.
//!
//! EmailConfig:
//! from_email: Forwarded emails will be received from this SES verified email address.
//! subject_prefix: Forwarded emails subject will contain this prefix.
//! email_bucket: S3 bucket to store raw SES emails.
//! email_key_prefix: S3 key prefix where SES stores emails.
//! allow_plus: Enables support for plus(+) sign suffixes on email addresses.
//!     Once this is set, the username is parsed to remove everything after the `+` sign
//!     Example: (
//!         jon+doe@doe.example,
//!         jon+test@doe.example
//!     ) - > will all be treated as => (
//!         jon@doe.example
//!     )
//! email_mapping: Dictionary mapping showing where to forward the emails
//!                 "sender": ["recipient", "emails"]
//!                To match all email addresses on a domain, use a key without the name part of an email address( `@example.com`)
//!                To match a mailbox on all domains, use a key without the `@` symbol e.g (`info`, 'admin')
mod config;
mod lib;
use std::env;
use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use lib::{LambdaRequest, LambdaResponse};
use log::{self, error};
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(my_handler);

    Ok(())
}

fn privatemail_handler(
    e: LambdaEvent,
    c: Context,
) -> Result<LambdaResponse, HandlerError> {
    if e.request_id == "" {
        error!("Empty request_id in request {}", c.aws_request_id);
        bail!("Empty first name");
    }

    Ok(CustomOutput { message: format!("Hello, {}!", e.first_name) })
}
