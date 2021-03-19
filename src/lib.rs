//!
//! A rust library for handling SNS requests to Lambda.
//!
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
mod config;

use lambda_runtime::{Context, Error};
use rusoto_core::Region;
use rusoto_ses::{
    Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient,
};
use serde::Serialize;
use serde_json::Value;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, process};
use tracing::log::LevelFilter;
use tracing::{error, info, warn};

/// LambdaResponse: The Outgoing response being passed by the Lambda
#[derive(Debug, Default, Clone, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LambdaResponse {
    /** is_base_64_encoded response field */
    is_base_64_encoded: bool,

    /** status_code for lambda response */
    status_code: u32,

    /** response headers for lambda response */
    headers: HashMap<String, String>,

    /** response body for LambdaResponse struct */
    body: String,
}

impl LambdaResponse {
    /**
     *  Given a status_code and response body a new LambdaResponse
     *  is returned to the calling function
     * */
    pub fn new(status_code: u32, body: String) -> Self {
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

impl fmt::Display for LambdaResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LambdaResponse: status_code: {}, body: {}",
            self.status_code,
            self.body.to_string()
        )
    }
}

/// PrivatEmail_Handler: processes incoming messages from SNS
/// and forwards to the appropriate recipient email
pub(crate) async fn privatemail_handler(
    event: Value,
    ctx: Context,
) -> Result<LambdaResponse, Error> {
    // Enable Cloudwatch error logging at runtime
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    info!("Event: {:#?}, Context: {:#?}", event.as_object().unwrap(), ctx);

    // create ses client
    let ses_client = SesClient::new(Region::UsEast1);

    // Initialize the PrivatEmailConfig object
    let email_config = config::PrivatEmailConfig::new_from_env();

    // Fetch request payload
    let sns_payload = event["Records"][0]["Sns"].as_object().unwrap();
    info!("Raw Email Info: {:?}", sns_payload);

    let sns_message: HashMap<String, Value> =
        serde_json::from_str(sns_payload["Message"].as_str().unwrap())?;
    info!("Parsed SES Message: {:#?}", sns_message);
    info!("Parsed SES Message Mail: {:#?}", sns_message["mail"]);
    info!("Parsed SES Message Receipt: {:#?}", sns_message["receipt"]);
    info!("Parsed SES Message content: {:#?}", sns_message["content"]);

    // skip spam messages
    let spam_verdict: String = serde_json::from_value(
        sns_message["receipt"]["spamVerdict"]["status"].clone(),
    )?;
    let virus_verdict: String = serde_json::from_value(
        sns_message["receipt"]["virus"]["status"].clone(),
    )?;
    if spam_verdict == "FAIL" || virus_verdict == "FAIL" {
        warn!("Message contains spam or virus, skipping!");
        process::exit(200);
        // Ok(LambdaResponse(200, "message skipped"))
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let original_sender: Vec<String> = serde_json::from_value(
        sns_message["mail"]["commonHeaders"]["replyTo"].clone(),
    )?;
    let subject: String = serde_json::from_value(
        sns_message["mail"]["commonHeaders"]["subject"].clone(),
    )?;
    let mail_content: String =
        serde_json::from_value(sns_message["content"].clone())?;
    let mail_content_txt: String =
        serde_json::from_value(sns_message["content"].clone())?;

    info!("sender: {:#?}", original_sender);
    info!("Subject: {:#?}", subject);
    info!("To Email: {:#?}", email_config.to_email.to_string());
    info!("Content: {:#?}", mail_content);

    let ses_email_message = SendEmailRequest {
        configuration_set_name: None,
        destination: Destination {
            bcc_addresses: None,
            cc_addresses: None,
            to_addresses: Some(vec![email_config.to_email.to_string()]),
        },
        message: Message {
            body: Body {
                html: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: mail_content,
                }),
                text: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: mail_content_txt,
                }),
            },
            subject: Content {
                charset: Some(String::from("utf-8")),
                data: subject,
            },
        },
        reply_to_addresses: Some(original_sender),
        return_path: None,
        return_path_arn: None,
        source: email_config.from_email.to_string(),
        source_arn: None,
        tags: None,
    };

    match ses_client.send_email(ses_email_message).await {
        Ok(email_response) => {
            info!("Email forward success: {:?}", email_response);
            Ok(LambdaResponse::new(200, email_response.message_id))
        }
        Err(error) => {
            error!("Error forwarding email: {:?}", error);
            Err(Box::new(error))
        }
    }
}

/** Test module for privatemail package */
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::{env, fs};

    fn read_test_event() -> Value {
        // Open the file in read-only mode with buffer.

        let mut srcdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        srcdir.push("tests/payload/testEvent.json");
        println!("Cur Dir: {}", srcdir.display());

        // Read the JSON contents of the file as an instance of `String`.
        let input_str = fs::read_to_string(srcdir.as_path()).unwrap();
        info!("Input str: {}", input_str);

        // Return the `Value`.
        return serde_json::from_str(input_str.as_str()).unwrap();
    }

    #[tokio::test]
    #[ignore = "skipping integration because because of IAM requirements"]
    async fn handler_handles() {
        env::set_var("TO_EMAIL", "test@nyah.dev");
        env::set_var("FROM_EMAIL", "achu@fufu.africa");
        let test_event = read_test_event();
        assert_eq!(
            privatemail_handler(test_event, Context::default())
                .await
                .expect("expected Ok(_) response")
                .status_code,
            200
        )
    }
}
