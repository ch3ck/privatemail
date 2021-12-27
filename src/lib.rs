//! -*- mode: rust; -*-
//!
//! This file is part of privatemail crate.
//! Copyright (c) 2021 Nyah Check
//! See LICENSE for licensing information.
//!
//! A rust library for handling SNS requests to Lambda.
//!
//! Authors:
//! - Nyah Check <hello@nyah.dev>
//!
//! Example:
//!
//! ```
//! use privatemail::PrivatEmailConfig;
//! use serde::{Deserialize, Serialize};
//!
//! async fn privatemail_handler() {
//!     // Initialize PrivatEmailConfig object.
//!     let email_config = PrivatEmailConfig::default();
//!
//!     // Get email recipient and process incoming mail
//!     ...
//!
//!     // Forward to recipent
//!     ...
//! }
//! ```
#![allow(clippy::field_reassign_with_default)]
use config::PrivatEmailConfig;

use aws_lambda_events::event::ses::SimpleEmailEvent;
use aws_lambda_events::event::sns::SnsEvent;
use lambda_runtime::{Context, Error};
use mailparse::*;
use rusoto_core::Region;
use rusoto_ses::{
    Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;

use tracing::{error, trace};

/// LambdaResponse: The Outgoing response being passed by the Lambda
#[derive(Debug, Default, Clone, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LambdaResponse {
    /// is_base_64_encoded response field
    is_base_64_encoded: bool,

    /// status_code for lambda response
    status_code: u32,

    /// response headers for lambda response
    headers: HashMap<String, String>,

    /// response body for LambdaResponse struct
    body: String,
}

impl LambdaResponse {
    /// Given a status_code and response body a new LambdaResponse
    /// is returned to the calling function
    pub fn new(status_code: u32, body: &str) -> Self {
        let mut header = HashMap::new();
        header.insert("content-type".to_owned(), "application/json".to_owned());
        LambdaResponse {
            is_base_64_encoded: false,
            status_code,
            headers: header,
            body: serde_json::to_string(&body).unwrap(),
        }
    }
}

impl std::fmt::Display for LambdaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LambdaResponse: status_code: {}, body: {}",
            self.status_code, self.body
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailContent {
    #[serde(rename = "notificationType")]
    notification_type: String,
    content: String,
}

/// PrivatEmail_Handler: processes incoming messages from SNS
/// and forwards to the appropriate recipient email
pub(crate) async fn privatemail_handler(
    event: SnsEvent,
    ctx: Context,
) -> Result<LambdaResponse, Error> {
    // install global collector configured based on RUST_LOG env var
    let xray_trace_id = &ctx.xray_trace_id.clone();
    env::set_var("_X_AMZN_TRACE_ID", xray_trace_id);

    // Enable Cloudwatch error logging at runtime
    trace!("Event: {:#?}, Context: {:#?}", event, ctx);

    // create ses client
    let ses_client = SesClient::new(Region::default());

    // Initialize the PrivatEmailConfig object
    let email_config = PrivatEmailConfig::new_from_env();

    // Fetch ses request payload from sns message
    let ses_mail: SimpleEmailEvent =
        serde_json::from_str(&event.records[0].sns.message.as_ref().unwrap())?;

    // skip spam messages
    let ses_receipt = &ses_mail.records[0].ses.receipt;
    if ses_receipt.spam_verdict.status.as_ref().unwrap() == "FAIL"
        || ses_receipt.virus_verdict.status.as_ref().unwrap() == "FAIL"
    {
        let err_msg = "Message contains spam or virus, skipping!";
        error!(err_msg);
        return Ok(LambdaResponse::new(200, err_msg));
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let original_sender: String = ses_mail.records[0]
        .ses
        .mail
        .common_headers
        .return_path
        .as_ref()
        .unwrap()
        .to_string();
    let subject: String = ses_mail.records[0]
        .ses
        .mail
        .common_headers
        .subject
        .as_ref()
        .unwrap()
        .to_string();

    // parse email content
    let raw_mail: EmailContent =
        serde_json::from_str(&event.records[0].sns.message.as_ref().unwrap())?;
    let parsed_mail = parse_mail(&raw_mail.content.as_bytes()).unwrap();
    let mail_content = parsed_mail.subparts[1].get_body_raw().unwrap();
    let msg_body = charset::decode_latin1(&mail_content).to_string();
    trace!("HTML content: {:#?}", mail_content);

    // Skip mail if it's from blacklisted email
    for email in
        email_config.black_list.unwrap_or_else(|| panic!("Missing black list"))
    {
        if !email.as_str().is_empty()
            && original_sender.contains(email.as_str())
        {
            let mut err_msg: String =
                "Message is from blacklisted email: ".to_owned();
            err_msg.push_str(email.as_str());
            trace!("`{}`, skipping!", err_msg.as_str());
            return Ok(LambdaResponse::new(200, err_msg.as_str()));
        }
    }

    let ses_email_message = SendEmailRequest {
        configuration_set_name: Default::default(),
        destination: Destination {
            bcc_addresses: Default::default(),
            cc_addresses: Default::default(),
            to_addresses: Some(vec![email_config.to_email.to_string()]),
        },
        message: Message {
            body: Body {
                html: Some(Content {
                    charset: Default::default(),
                    data: msg_body,
                }),
                text: Default::default(),
            },
            subject: Content { charset: Default::default(), data: subject },
        },
        reply_to_addresses: Some(vec![original_sender]),
        return_path: Default::default(),
        return_path_arn: Default::default(),
        source: email_config.from_email.to_string(),
        source_arn: Default::default(),
        tags: Default::default(),
    };

    match ses_client.send_email(ses_email_message).await {
        Ok(email_response) => {
            trace!("Email forward success: {:?}", email_response);
            Ok(LambdaResponse::new(200, &email_response.message_id))
        }
        Err(error) => {
            tracing::error!("Error forwarding email: {:?}", error);
            Err(Box::new(error))
        }
    }
}

/// Test module for privatemail package
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::path::PathBuf;
    use std::{env, fs};

    fn read_test_event(file_name: String) -> SnsEvent {
        // Open the file in read-only mode with buffer.

        let mut srcdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut file_dir: String = "tests/payload/".to_owned();
        file_dir.push_str(file_name.as_str());
        srcdir.push(file_dir.as_str());
        println!("Cur Dir: {}", srcdir.display());

        // Read the JSON contents of the file as an instance of `String`.
        let input_str = fs::read_to_string(srcdir.as_path()).unwrap();
        trace!("Input str: {}", input_str);

        // Return the `Value`.
        return serde_json::from_str(input_str.as_str()).unwrap();
    }

    #[tokio::test]
    // #[ignore = "skipping integration because of IAM requirements"]
    async fn handler_with_success() {
        env::set_var("TO_EMAIL", "nyah@hey.com");
        env::set_var("FROM_EMAIL", "test@nyah.dev");
        let test_event = read_test_event(String::from("test_event.json"));

        assert_eq!(
            privatemail_handler(test_event, Context::default())
                .await
                .expect("expected Ok(_) response")
                .status_code,
            200
        )
    }

    #[tokio::test]
    #[ignore = "skipping integration because of IAM requirements"]
    async fn handler_with_black_listed_email() {
        env::set_var("TO_EMAIL", "test@nyah.dev");
        env::set_var("FROM_EMAIL", "fufu@achu.soup");
        env::set_var("BLACK_LIST", "achu.soup");
        let test_event = read_test_event(String::from("test_event.json"));

        assert_eq!(
            privatemail_handler(test_event, Context::default())
                .await
                .expect("expected Ok(_) response")
                .status_code,
            200
        )
    }
}
