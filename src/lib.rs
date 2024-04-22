//! -*- mode: rust; -*-
//!
//! This file is part of privatemail crate.
//! Copyright (c) 2022 Nyah Check
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
//! use crate::lib::config::PrivatEmailConfig;
//! use serde::{Deserialize, Serialize};
//!
//! async fn privatemail_handler() {
//!     // Initialize PrivatEmailConfig object.
//!     let email_config = PrivatEmailConfig::default();
//!
//! }
//! ```

#![forbid(unsafe_code)]
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod config;

use config::PrivatEmailConfig;
use lambda_runtime::{Error, LambdaEvent};
use mailparse::parse_mail;
use rusoto_core::Region;
use rusoto_ses::{
    Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, fmt::Debug};
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
pub struct EmailReceiptNotification {
    #[serde(rename = "notificationType")]
    notification_type: String,
    mail: Mail,
    receipt: Receipt,
    content: String,
    // #[serde(flatten)]
    // other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mail {
    timestamp: String,
    source: String,
    #[serde(rename = "messageId")]
    message_id: String,
    destination: Vec<String>,

    #[serde(rename = "commonHeaders")]
    common_headers: CommonHeaders,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonHeaders {
    // replyTo: Vec<String>,
    subject: String,
    #[serde(rename = "returnPath")]
    return_path: String,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Receipt {
    #[serde(rename = "spamVerdict")]
    spam_verdict: Verdict,
    #[serde(rename = "virusVerdict")]
    virus_verdict: Verdict,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Verdict {
    status: String,
}

/// PrivatEmail_Handler: processes incoming messages from SNS
/// and forwards to the appropriate recipient email
pub async fn privatemail_handler(
    lambda_event: LambdaEvent<Value>,
) -> Result<LambdaResponse, Error> {
    let (event, ctx) = lambda_event.into_parts();

    // install global collector configured based on RUST_LOG env var
    let xray_trace_id = &ctx.xray_trace_id.as_ref().unwrap();
    env::set_var("_X_AMZN_TRACE_ID", xray_trace_id);

    // Enable Cloudwatch error logging at runtime
    trace!("Event: {:#?}, Context: {:#?}", event, ctx);

    // create ses client
    let ses_client = SesClient::new(Region::default());

    // Initialize the PrivatEmailConfig object
    let email_config = PrivatEmailConfig::new_from_env();

    // fetch sns payload
    let sns_payload = event["Records"][0]["Sns"]
        .as_object()
        .unwrap_or_else(|| panic!("Missing sns payload"));
    tracing::info!("Raw Email Info: {:?}", sns_payload);

    // Fetch request payload
    let sns_payload = event["Records"][0]["Sns"]
        .as_object()
        .unwrap_or_else(|| panic!("Missing sns payload"));
    tracing::info!("Raw Email Info: {:?}", sns_payload);

    // Fetch ses request payload from sns message
    let ses_mail: EmailReceiptNotification = serde_json::from_str(
        sns_payload["Message"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing Message field")),
    )?;

    // skip spam messages
    let ses_receipt = &ses_mail.receipt;
    if ses_receipt.spam_verdict.status == "FAIL"
        || ses_receipt.virus_verdict.status == "FAIL"
    {
        let err_msg = "Message contains spam or virus, skipping!";
        error!(err_msg);
        return Ok(LambdaResponse::new(200, err_msg));
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let original_sender: String =
        ses_mail.mail.common_headers.return_path.to_string();
    let subject: String = ses_mail.mail.common_headers.subject.to_string();

    // parse email content
    let mail = parse_mail(ses_mail.content.as_bytes()).unwrap();
    let content = mail.subparts[1].get_body_raw().unwrap();
    let msg_body = charset::decode_latin1(&content).to_string();
    trace!("HTML content: {:#?}", content);

    // Skip mail if it's from blacklisted email
    for email in
        email_config.black_list.unwrap_or_else(|| panic!("Missing black list"))
    {
        if !email.is_empty()
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
    use lambda_runtime::Context;
    use std::fs;
    use std::path::PathBuf;

    fn read_test_event(file_name: String) -> Value {
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
    #[ignore = "skipping integration because of IAM requirements"]
    async fn handler_with_success() {
        env::set_var("TO_EMAIL", "nyah@hey.com");
        env::set_var("FROM_EMAIL", "test@nyah.dev");
        let test_event = read_test_event(String::from("test_event.json"));

        assert_eq!(
            privatemail_handler(LambdaEvent {
                payload: test_event,
                context: Context::default()
            })
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
            privatemail_handler(LambdaEvent {
                payload: test_event,
                context: Context::default()
            })
            .await
            .expect("expected Ok(_) response")
            .status_code,
            200
        )
    }
}
