//! Copyright 2021 Nyah Check crate.
//!
//! Application-specific configuration for PrivatEmail
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

impl Default for PrivatEmailConfig {
    fn default() -> Self {
        PrivatEmailConfig {
            from_email: String::from("nyah.dev"),
            to_email: String::from("nyah@hey.com"),
            subject_prefix: String::from("PrivateMail: "), // not currently used
            email_bucket: String::from("nyah-ses-emails"), // not currently used
            email_key_prefix: String::from("nyah/"), // not currently used
        }
    }
}
