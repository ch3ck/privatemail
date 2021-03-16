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
//use self::PrivatEmailConfig;

use bytes::Bytes;
use lambda_runtime::{Context, Error};
use regex::Regex;
use rusoto_core::Region;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
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
    info!("Email request: {:#?}", sns_payload.as_str());

    let raw_email_info = sns_payload.get("Records").unwrap();
    info!("Raw Email Info: {:?}", raw_email_info);

    let email_info = raw_email_info
        .get(0)
        .unwrap()
        .get("Sns")
        .unwrap()
        .get("Message")
        .unwrap();

    // skip spam messages
    if email_info
        .get("receipt")
        .unwrap()
        .get("spamVerdict")
        .unwrap()
        .get("status")
        .unwrap()
        == "FAIL"
        || email_info
            .get("receipt")
            .unwrap()
            .get("virusVerdict")
            .unwrap()
            .get("status")
            .unwrap()
            == "FAIL"
    {
        warn!("Message contains spam or virus, skipping!");
        process::exit(200);
        // Ok(LambdaResponse(200, "message skipped"))
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let from_header = email_info
        .get("mail")
        .unwrap()
        .get("commonHeaders")
        .unwrap()
        .get("from")
        .unwrap()
        .get(0)
        .unwrap();
    info!("FromHeader: {:#?}", from_header);

    let new_from_header = from_header.as_str().unwrap().replace("/<(.*)>/", "");
    let final_from_header =
        format!("{} < {} > ", new_from_header, email_config.from_email);
    info!("FinalFromHeader: {}", final_from_header);

    // extract email content
    let email_message = email_info.get("content").unwrap().as_str().unwrap();
    let mut new_message_header = format!(
        "From: {final_from}\r\nReply-To: {from}\r\nX-Original-To: {to}\r\nTo: {to}\r\n",
        final_from=final_from_header, from=email_info.get("info").unwrap().get("commonHeaders").unwrap().get("from").unwrap().get(0).unwrap(),
        to=email_info.get("info").unwrap().get("commonHeaders").unwrap().get("to").unwrap().get(0).unwrap(),
    );

    // Check if other emails are cc'ed
    if email_info
        .get("mail")
        .unwrap()
        .get("commonHeaders")
        .unwrap()
        .get("cc")
        .unwrap()
        .is_string()
    {
        let cc_list = format!(
            "CC: {}\r\n",
            email_info
                .get("info")
                .unwrap()
                .get("commonHeaders")
                .unwrap()
                .get("cc")
                .unwrap()
        );
        new_message_header.push_str(cc_list.as_str());
    }

    // Add subject to email
    let subject = format!(
        "Subject: {}\r\n",
        email_info
            .get("info")
            .unwrap()
            .get("commonHeaders")
            .unwrap()
            .get("subject")
            .unwrap()
    );
    new_message_header.push_str(subject.as_str());
    info!("Final Email Headers: {}", new_message_header);

    let mut final_raw_email: String = String::new();

    // Add formatting fixes to email content
    if email_message.is_empty() {
        email_message.to_string().push_str(
            format!("{}\r\n No message!", new_message_header).as_str(),
        );
    } else {
        let mut re = Regex::new(r"/Content-Type:.+\s *boundary.*/").unwrap();
        match re.captures(&email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => {
                let renone = Regex::new(r"/^Content-Type:(.*)/m").unwrap();
                match renone.captures(&email_message) {
                    Some(x) => email_message.to_string().push_str(
                        format!("{}\r\n", x.get(0).map_or("", |m| m.as_str()))
                            .as_str(),
                    ),
                    None => unreachable!(),
                }
            }
        }

        re = Regex::new(r"/^Content-Transfer-Encoding:(.*)/m").unwrap();
        match re.captures(&email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => unreachable!(),
        }

        re = Regex::new(r"/^MIME-Version:(.*)/m").unwrap();
        match re.captures(&email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => unreachable!(),
        }

        // cleanup email message and append headers
        let str_list = email_message.split("\r\n\r\n");
        let mut str_vector: Vec<&str> = str_list.collect();
        str_vector.remove(0);
        final_raw_email = format!(
            "{}\r\n{}",
            new_message_header.as_str(),
            str_vector.join("\r\n\r\n")
        );
    }

    // Forward raw email to recipient address
    let raw_email = SendRawEmailRequest {
        raw_message: RawMessage { data: Bytes::from(final_raw_email) },
        destinations: Some(vec![email_config.to_email]),
        source: Some(email_config.from_email),
        configuration_set_name: None,
        from_arn: None,
        return_path_arn: None,
        source_arn: None,
        tags: None,
    };

    match ses_client.send_raw_email(raw_email).await {
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
    #[ignore]
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
