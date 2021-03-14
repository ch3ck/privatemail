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
use lambda_runtime::{handler_fn, Context, Error};
use tracing::log::LevelFilter;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;


#[derive(Deserialize)]
struct Request {
    command: String,
}


#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: Request, ctx: Context) -> Result<Response, Error> {
    // extract some useful info from the request
    let command = event.command;

    // prepare the response
    let resp = Response {
        req_id: ctx.request_id,
        msg: format!("Command {} executed.", command),
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}
