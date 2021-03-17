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
use serde_json::Value;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, process};
use tracing::log::LevelFilter;
use tracing::{error, info, warn};

// LambdaRequest: Represents the incoming Request from AWS Lambda
//                This is deserialized into a struct payload
//
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LambdaRequest {
    /** lambda request body */
    body: Value,
}

/// LambdaResponse: The Outgoing response being passed by the Lambda
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaResponse {
    /** is_base_64_encoded response field */
    is_base_64_encoded: bool,

    /** status_code for lambda response */
    status_code: u32,

    /** response headers for lambda response */
    headers: HashMap<String, String>,

    /** response body for LambdaResponse struct */
    #[serde(default, skip_serializing_if = "String::is_empty")]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MessageHeader {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    name: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    value: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptStatus {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    status: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LambdaInfo {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    r#type: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    topic_arn: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    function_arn: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    invocation_type: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptInfo {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    timestamp: String,

    processing_time_millis: u32,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recipients: Vec<String>,

    spam_verdict: ReceiptStatus,
    virus_verdict: ReceiptStatus,
    spf_verdict: ReceiptStatus,
    dkim_verdict: ReceiptStatus,
    dmarc_verdict: ReceiptStatus,
    action: LambdaInfo,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommonHeaderReq {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    return_path: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    from: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    date: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    to: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    message_id: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    subject: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MailInfoReq {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    timestamp: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    source: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    message_id: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    destination: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    headers_truncated: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    headers: Vec<MessageHeader>,
    common_headers: CommonHeaderReq,
}

/// SesMessageRequest: SES Message
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SesMessageRequest {
    /** notification type */
    notification_type: String,

    /** receipt metadata **/
    receipt: ReceiptInfo,

    /** Email content */
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
    let sns_payload: Value = serde_json::from_value(event.body.into()).unwrap();
    // info!("Email request: {:#?}", sns_payload.as_str());

    let raw_email_info = sns_payload.get("Records").unwrap();
    // info!("Raw Email Info: {:?}", raw_email_info);

    let email_info = raw_email_info
        .get(0)
        .unwrap()
        .get("Sns")
        .unwrap()
        .get("Message")
        .unwrap()
        .to_owned();

    info!("Email Message: {:?}", email_info);
    let new_email_info: SesMessageRequest =
        serde_json::from_value(email_info).unwrap();
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
            bcc_addresses: Some(new_email_info.mail.common_headers.bcc),
            cc_addresses: Some(new_email_info.mail.common_headers.cc),
            to_addresses: Some(vec![email_config.to_email]),
        },
        message: Message {
            body: Body {
                html: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: new_email_info.content,
                }),
                text: Some(Content {
                    charset: Some(String::from("utf-8")),
                    data: new_email_info.content.clone(),
                }),
            },
            subject: Content {
                charset: Some(String::from("utf-8")),
                data: new_email_info.mail.common_headers.subject,
            },
        },
        reply_to_addresses: Some(new_email_info.mail.common_headers.from),
        return_path: Some(new_email_info.mail.source),
        return_path_arn: None,
        source: new_email_info.mail.source,
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
        Ok(LambdaRequest { body: req })
    }

    #[tokio::test]
    // #[ignore]
    async fn handler_handles() {
        let test_event = read_test_event().unwrap();
        assert_eq!(
            privatemail_handler(test_event, Context::default())
                .await
                .expect("expected Ok(_) value")
                .status_code,
            200
        )
    }
}
