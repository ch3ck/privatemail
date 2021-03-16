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
use config::PrivatEmailConfig;

use bytes::Bytes;
use lambda_runtime::{Context, Error};
use regex::Regex;
use rusoto_core::Region;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use tracing::log::LevelFilter;
use tracing::{error, info, warn};

// LambdaRequest: Represents the incoming Request from AWS Lambda
//                This is deserialized into a struct payload
//
#[derive(Debug, Clone, Default, Serialize)]
#[serde(default)]
pub struct LambdaRequest<Data: DeserializeOwned> {
    #[serde(deserialize_with = "deserializer")]
    /** lambda request body */
    body: Data,
}

impl<Data: DeserializeOwned> LambdaRequest<Data> {
    pub fn body(&self) -> Data {
        self.body
    }
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
pub(crate) async fn PrivatEmail_Handler(
    event: LambdaRequest<Value>,
    ctx: Context,
) -> Result<LambdaResponse, Error> {
    // Enable Cloudwatch error logging at runtime
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    // create ses client
    let ses_client = SesClient::new(Region::UsEast1);

    // Initialize the PrivatEmailConfig object
    let email_config = PrivatEmailConfig::new_from_env();

    // Fetch request payload
    let sns_payload = event.body();
    info!("Email request: {:#?}", sns_payload.as_str());

    let raw_email_info: Value =
        serde_json::from_value(sns_payload.into()).unwrap();
    info!("Raw Email Info: {:?}", raw_email_info);

    let email_info =
        raw_email_info["Records"][0]["Sns"]["Message"].as_object().unwrap();

    // skip spam messages
    if email_info["receipt"]["spamVerdict"]["status"].as_str().unwrap()
        == "FAIL"
        || email_info["receipt"]["virusVerdict"]["status"].as_str().unwrap()
            == "FAIL"
    {
        warn!("Message contains spam or virus, skipping!");
        // Ok(LambdaResponse(200, "message skipped"))
    }

    // Rewrite Email From header to contain sender's name with forwarder's email address
    let mut from_header =
        email_info["mail"]["commonHeaders"]["from"][0].as_str().unwrap();
    info!("FromHeader: {:#?}", from_header);

    from_header = &str::replace(from_header, "/<(.*)>/", "");
    let final_from_header = format!(" < {} > ", email_config.from_email);
    info!("FinalFromHeader: {}", final_from_header);

    // extract email content
    let mut email_message = email_info["content"].as_str().unwrap();
    let mut new_message_header = format!(
        "From: {final_from}\r\nReply-To: {from}\r\nX-Original-To: {to}\r\nTo: {to}\r\n",
        final_from=final_from_header, from=email_info["info"]["commonHeaders"]["from"][0].as_str().unwrap(),
        to=email_info["info"]["commonHeaders"]["to"][0].as_str().unwrap(),
    );

    // Check if other emails are cc'ed
    if email_info["mail"]["commonHeaders"]["cc"].is_string() {
        let cc_list = format!(
            "CC: {}\r\n",
            email_info["info"]["commonHeaders"]["cc"].as_str().unwrap()
        );
        new_message_header.push_str(cc_list.as_str());
    }

    // Add subject to email
    let subject = format!(
        "Subject: {}\r\n",
        email_info["info"]["commonHeaders"]["subject"].as_str().unwrap()
    );
    new_message_header.push_str(subject.as_str());
    info!("Final Email Headers: {}", new_message_header);

    // Add formatting fixes to email content
    if email_message.is_empty() {
        email_message.to_string().push_str(
            format!("{}\r\n No message!", new_message_header).as_str(),
        );
    } else {
        let mut re = Regex::new(r"/Content-Type:.+\s *boundary.*/").unwrap();
        match re.captures(email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => {
                let renone = Regex::new(r"/^Content-Type:(.*)/m").unwrap();
                match renone.captures(email_message) {
                    Some(x) => email_message.to_string().push_str(
                        format!("{}\r\n", x.get(0).map_or("", |m| m.as_str()))
                            .as_str(),
                    ),
                    None => unreachable!(),
                }
            }
        }

        re = Regex::new(r"/^Content-Transfer-Encoding:(.*)/m").unwrap();
        match re.captures(email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => unreachable!(),
        }

        re = Regex::new(r"/^MIME-Version:(.*)/m").unwrap();
        match re.captures(email_message) {
            Some(res) => email_message.to_string().push_str(
                format!("{}\r\n", res.get(0).map_or("", |m| m.as_str()))
                    .as_str(),
            ),
            None => unreachable!(),
        }

        // cleanup email message and append headers
        let mut str_list = email_message.to_string().split("\r\n\r\n");
        let mut str_vector: Vec<&str> = str_list.collect();
        str_vector.remove(0);
        email_message = format!(
            "{}\r\n{}",
            new_message_header.as_str(),
            str_vector.join("\r\n\r\n")
        )
        .as_str();
    }

    // Forward raw email to recipient address
    let mut raw_email = SendRawEmailRequest {
        raw_message: RawMessage { data: Bytes::from(email_message) },
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

    fn read_test_event() -> Result<LambdaRequest<Value>, Error> {
        // Open the file in read-only mode with buffer.

        let srcdir = PathBuf::from("./src");
        let mut fp =
            srcdir.parent().unwrap().join("/tests/payload/testEvent.json");
        let file = File::open(fp)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let req = serde_json::from_reader(reader)?;

        // Return the `LambdaRequest`.
        Ok(LambdaRequest { body: req })
    }

    #[tokio::test]
    async fn handler_handles() {
        let test_event = read_test_event().unwrap();
        assert_eq!(
            PrivatEmail_Handler(test_event, Context::default())
                .await
                .expect("expected Ok(_) value")
                .status_code,
            200
        )
    }
}
