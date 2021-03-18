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
use serde::{Deserialize, Serialize};
// use serde_json::Value;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, process};
use tracing::log::LevelFilter;
use tracing::{error, info, warn};

// LambdaRequest: Represents the incoming Request from AWS Lambda
//                This is deserialized into a struct payload
//
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LambdaRequest {
    /** lambda request body */
    records: Vec<LambdaRequestRecord>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LambdaRequestRecord {
    /** event source */
    event_source: String,

    /** event version */
    event_version: String,

    /** event subscription arn*/
    event_subscription_arn: String,

    /** SNS Message body */
    sns: SNSMessageBody,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SNSMessageBody {
    r#type: String,

    message_id: String,

    topic_arn: String,

    subject: String,

    /** SES Message request */
    message: SesMessageRequest,

    timestamp: String,

    signature_version: u32,

    signature: String,

    signing_cert_url: String,

    unsubscribe_url: String,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    message_attributes: HashMap<String, String>,
}

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

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MessageHeader {
    name: String,

    value: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ReceiptStatus {
    status: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct LambdaInfo {
    r#type: String,

    topic_arn: String,

    function_arn: String,

    invocation_type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ReceiptInfo {
    timestamp: String,

    processing_time_millis: u32,

    recipients: Vec<String>,

    spam_verdict: ReceiptStatus,

    virus_verdict: ReceiptStatus,

    spf_verdict: ReceiptStatus,
    dkim_verdict: ReceiptStatus,
    dmarc_verdict: ReceiptStatus,
    action: LambdaInfo,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CommonHeaderReq {
    return_path: String,

    from: Vec<String>,

    date: String,

    to: Vec<String>,

    cc: Vec<String>,

    bcc: Vec<String>,

    message_id: String,

    subject: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MailInfoReq {
    timestamp: String,

    source: String,

    message_id: String,

    destination: Vec<String>,

    headers_truncated: bool,

    headers: Vec<MessageHeader>,
    common_headers: CommonHeaderReq,
}

/// SesMessageRequest: SES Message
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SesMessageRequest {
    /** notification type */
    notification_type: String,

    /** receipt metadata **/
    receipt: ReceiptInfo,

    /** Email content */
    content: String,

    /** Email metadata */
    mail: MailInfoReq,
}

/// PrivatEmail_Handler: processes incoming messages from SNS
/// and forwards to the appropriate recipient email
pub(crate) async fn privatemail_handler(
    event: LambdaRequest,
    ctx: Context,
) -> Result<LambdaResponse, Error> {
    // Enable Cloudwatch error logging at runtime
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    info!("Event: {:#?}, Context: {:#?}", event, ctx);

    // create ses client
    let ses_client = SesClient::new(Region::UsEast1);

    // Initialize the PrivatEmailConfig object
    let email_config = config::PrivatEmailConfig::new_from_env();

    // Fetch request payload
    let sns_payload: &LambdaRequestRecord = event.records.first().unwrap();
    info!("Raw Email Info: {:?}", sns_payload);

    let email_info = &sns_payload.sns.message;
    info!("Email Message: {:?}", email_info);
    let new_email_info: &SesMessageRequest = email_info;
    info!("Email NotificationType: {:#?}", new_email_info);

    // skip spam messages
    if new_email_info.receipt.spam_verdict.status == "FAIL"
        || new_email_info.receipt.virus_verdict.status == "FAIL"
    {
        warn!("Message contains spam or virus, skipping!");
        process::exit(200);
        // Ok(LambdaResponse(200, "message skipped"))
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let ses_email_message = SendEmailRequest {
        configuration_set_name: None,
        destination: Destination {
            bcc_addresses: Some(new_email_info.mail.common_headers.bcc.clone()),
            cc_addresses: Some(new_email_info.mail.common_headers.cc.clone()),
            to_addresses: Some(vec![email_config.to_email.clone()]),
        },
        message: Message {
            body: Body {
                html: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: new_email_info.content.to_string(),
                }),
                text: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: new_email_info.content.to_string(),
                }),
            },
            subject: Content {
                charset: Some(String::from("utf-8")),
                data: new_email_info.mail.common_headers.subject.clone(),
            },
        },
        reply_to_addresses: Some(
            new_email_info.mail.common_headers.from.clone(),
        ),
        return_path: Some(new_email_info.mail.source.to_string()),
        return_path_arn: None,
        source: new_email_info.mail.source.clone(),
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
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    fn read_test_event() -> Result<LambdaRequest, Error> {
        // Open the file in read-only mode with buffer.

        let mut srcdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        srcdir.push("tests/payload/testEvent.json");
        println!("Cur Dir: {}", srcdir.display());
        let file = File::open(srcdir)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let req = serde_json::from_reader(reader)?;

        // Return the `LambdaRequest`.
        Ok(req)
    }

    #[tokio::test]
    // #[ignore]
    async fn handler_handles() {
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
