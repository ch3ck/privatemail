//! Copyright 2021 Nyah Check crate.
//!
//! Application-specific configuration for PrivatEmail

#![allow(clippy::style)]
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

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
    pub from_email: String,
    /** Recipient email address that receives the forwarded SES email */
    pub to_email: String,
    /** Forwarded emails subject will contain this prefix */
    pub subject_prefix: String,
    /** S3 bucket to store raw SES emails */
    pub email_bucket: String,
    /** S3 key prefix where SES stores emails */
    pub email_key_prefix: String,
}

/// Create a new PrivatEmailConfig client struct from environment variables.
impl PrivatEmailConfig {

    /// Create default method for PrivatEmailConfig struct
    pub fn default() -> Self {
        PrivatEmailConfig {
            from_email: String::from("nyah.dev"),
            to_email: String::from("nyah@hey.com"),
            subject_prefix: String::from("PrivateMail: "), // not currently used
            email_bucket: String::from("nyah-ses-emails"), // not currently used
            email_key_prefix: String::from("nyah/"), // not currently used
        }
    }

    /// Create new PrivatEmailConfig struct from environment variables.
    /// As long as you have the `from_email` and `to_email` environment setup; this should work
    pub fn new_from_env() -> Self {
        PrivatEmailConfig {
            from_email: env::var("from_email").unwrap(),
            to_email: env::var("to_email").unwrap(),
            subject_prefix: env::var("subject_prefix").unwrap_or_default(),
            email_bucket: env::var("email_bucket").unwrap_or_default(),
            email_key_prefix: env::var("email_key_prefix").unwrap_or_default(),
        }
    }

    /// Create a new PrivatEmailConfig struct.PrivatEmailConfig
    /// You can leave the s3 bucket related fields empty since it's not currently being used
    pub fn new<T, T, S>(from_email: F, to_email: T, subject_prefix: S) -> Self
    where
        T: ToString,
        T: ToString,
        S: ToString,
    {
        PrivatEmailConfig {
            from_email: from_email,
            to_email: to_email,
            subject_prefix: subject_prefix,
            email_bucket: String::from(""),
            email_key_prefix: String::from(""),
        }
    }
}
