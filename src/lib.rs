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
use config::PrivatEmailConfig;

use lambda_runtime::{Context, Error};
use mailparse::*;
use rusoto_core::Region;
use rusoto_ses::{
    Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, process};

use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

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

impl fmt::Display for LambdaResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
pub(crate) async fn privatemail_handler(
    event: Value,
    ctx: Context,
) -> Result<LambdaResponse, Error> {
    // install global collector configured based on RUST_LOG env var
    let subscriber = FmtSubscriber::builder()
        // filter events with level TRACE or higher
        .with_max_level(Level::TRACE)
        // build without subscriber
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subsciber failed");

    // Enable Cloudwatch error logging at runtime
    info!(
        "Event: {:#?}, Context: {:#?}",
        event.as_object().unwrap_or_else(|| panic!("Missing event object")),
        ctx
    );

    // create ses client
    let ses_client = SesClient::new(Region::default());

    // Initialize the PrivatEmailConfig object
    let email_config = PrivatEmailConfig::new_from_env();

    // Fetch request payload
    let sns_payload = event["Records"][0]["Sns"]
        .as_object()
        .unwrap_or_else(|| panic!("Missing sns payload"));
    // info!("Raw Email Info: {:?}", sns_payload);

    let sns_message: EmailReceiptNotification = serde_json::from_str(
        sns_payload["Message"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing Message field")),
    )?;

    // skip spam messages
    if sns_message.receipt.spam_verdict.status == "FAIL"
        || sns_message.receipt.virus_verdict.status == "FAIL"
    {
        info!("Message contains spam or virus, skipping!");
        process::exit(200);
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let original_sender: String = sns_message.mail.common_headers.return_path;
    let subject: String = sns_message.mail.common_headers.subject;

    let parsed_mail = parse_mail(&sns_message.content.as_bytes()).unwrap();
    let mail_content: String = parsed_mail.subparts[1].get_body().unwrap();
    let mail_txt: String = parsed_mail.subparts[0].get_body().unwrap();
    info!("Sender: {:#?}", original_sender);
    info!("Subject: {:#?}", subject);
    info!("To Email: {:#?}", email_config.to_email.to_string());
    info!("Black List: {:#?}", email_config.black_list);
    info!("Content: {:#?}", mail_content);

    // Skip mail if it's from blacklisted email
    for email in
        email_config.black_list.unwrap_or_else(|| panic!("Missing black list"))
    {
        if original_sender.contains(email.as_str()) {
            info!("Message is from Blacklisted email: `{}`, skipping!", email);
            process::exit(200);
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
                    charset: Some(String::from("utf-8")),
                    data: mail_content,
                }),
                text: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: mail_txt,
                }),
            },
            subject: Content {
                charset: Some(String::from("utf-8")),
                data: subject,
            },
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
            info!("Email forward success: {:?}", email_response);
            Ok(LambdaResponse::new(200, &email_response.message_id))
        }
        Err(error) => {
            error!("Error forwarding email: {:?}", error);
            Err(Box::new(error))
        }
    }
}

/// Test module for privatemail package
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::{env, fs};

    fn read_test_event(file_name: String) -> Value {
        // Open the file in read-only mode with buffer.

        let mut srcdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut file_dir: String = "tests/payload/".to_owned();
        file_dir.push_str(file_name.as_str());
        srcdir.push(file_dir.as_str());
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
        env::set_var("TO_EMAIL", "onions@suya.io");
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
    #[ignore = "skipping integration because because of IAM requirements"]
    async fn handler_with_black_listed_email() {
        env::set_var("TO_EMAIL", "onions@suya.io");
        env::set_var("FROM_EMAIL", "test@nyah.dev");
        env::set_var("BLACK_LIST", "fufu.soup");
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
