//! Copyright 2021 Nyah Check crate.
//!
//! Application-specific configuration for PrivatEmail

#![allow(clippy::style)]
use serde::{Deserialize, Serialize};
use std::env;

/**
 * Config object for PrivatEmail.
 *
 * Implements [`serde::Deserialize`] and [`serde::Serialize`] and
 * can be composed with other consumer configs.
 * Example:
 *  Here is a custom `PrivatEmailConfig` for an application
 *
 * ```
 * use privatemail::PrivatEmailConfig;
 * use serde::Deserialize;
 *
 * #[derive(Deserialize)]
 * struct PrivatEmailConfig {
 *     privatemail_config: PrivatEmailConfig,
 *     /* other application related configs */
 * }
 *
 * fn main() -> Result<(), String> {
 *     let my_privatemail_config: PrivatEmailConfig = toml::from_str(
 *         r##"
 *              [email_server]
 *              from_email = "doe.example"
 *              to_email   = "recipient@mail.box"
 *              
 *
 *             ## ... (other app-specific config)
 *         "##
 *     ).map_err(|error| format!("parsing server config: {}", error))?;
 *
 *    let mail_config: &PrivatEmailConfig = &my_privatemail_config.from_email;
 *    /** privatemail_handler(mail_config) */
 *    Ok(())
 * ```
 */
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct PrivatEmailConfig {
    /** Forwarded emails will be received from this SES verified email address */
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub from_email: String,
    /** Recipient email address that receives the forwarded SES email */
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub to_email: String,
    /** Forwarded emails subject will contain this prefix */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_prefix: Option<String>,
    /** S3 bucket to store raw SES emails */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_bucket: Option<String>,
    /** S3 key prefix where SES stores emails */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_key_prefix: Option<String>,
}

/// Create default method for PrivatEmailConfig struct
impl Default for PrivatEmailConfig {
    fn default() -> Self {
        PrivatEmailConfig {
            from_email: String::from("nyah.dev"),
            to_email: String::from("nyah@hey.com"),
            subject_prefix: None, // not currently used
            email_bucket: None,
            email_key_prefix: None,
        }
    }
}

/// Create a new PrivatEmailConfig client struct from environment variables.
impl PrivatEmailConfig {
    /// Create new PrivatEmailConfig struct from environment variables.
    /// As long as you have the `from_email` and `to_email` environment setup; this should work
    pub fn new_from_env() -> Self {
        PrivatEmailConfig {
            from_email: env::var("from_email").unwrap(),
            to_email: env::var("to_email").unwrap(),
            subject_prefix: Some(String::from("PrivateMail: ")), // not currently used
            email_bucket: None,
            email_key_prefix: None,
        }
    }

    /// Create a new PrivatEmailConfig struct.PrivatEmailConfig
    /// You can leave the s3 bucket related fields empty since it's not currently being used
    pub fn new<F, T, S>(from_email: F, to_email: T, subject_prefix: S) -> Self
    where
        F: ToString,
        T: ToString,
        S: ToString,
    {
        PrivatEmailConfig {
            from_email: from_email.to_string(),
            to_email: to_email.to_string(),
            subject_prefix: Some(subject_prefix.to_string()),
            email_bucket: None,
            email_key_prefix: None,
        }
    }
}
