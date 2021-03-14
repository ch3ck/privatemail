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
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;

/** Test module for privatemail package */
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// LambdaRequest: Represents the incoming Request from AWS Lambda
//                This is deserialized into a struct payload
//
#[derive(Serialize, Debug)]
pub struct LambdaRequest<Data: DeserializeOwned> {
    #[serde(deserialize_with = "deserialize")]
    /** lambda request body */
    body: Data,
}

impl<Data: DeserializeOwned> LambdaRequest<Data> {
    pub fn body(&self) -> &Data {
        &self.body
    }
}

/// LambdaResponse: The Outgoing response being passed by the Lambda
#[derive(Serialize, Debug)]
pub struct LambdaResponse {
    #[serde(rename = "isBase64Encoded")]
    /** is_base_64_encoded response field */
    is_base_64_encoded: bool,

    #[serde(rename = "statusCode")]
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
