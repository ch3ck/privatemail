//! Copyright 2021 Nyah Check crate.
//! 
//! Application-specific configuration for PrivatEmail
use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;

//!
//! Configuration for a PrivatEmail server for SES.
//!
//! This type implements [`serde::Deserialize`] and [`serde::Serialize`] and it
//! can be composed with the consumer's configuration (whatever format that's
//! in).  For example, consumers could define a custom `AppConfig` for an app
//! that contains uses PrivatEmail server:
//!
//! ```
//! use privatemail::EmailConfig;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct AppConfig {
//!     email_server: EmailConfig,
//!     /* other app related configs */
//! }
//!
//! fn main() -> Result<(), String> {
//!     let my_config: EmailConfig = toml::from_str(
//!         r##"
//!             [email_server]
//!
//!             ## ... (other app-specific config)
//!         "##
//!     ).map_err(|error| format!("parsing server config: {}", error))?;
//! ```
//!
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct EmailConfig {
    /** Forwarded emails will be received from this SES verified email address */
    pub from_email: String,
    /** Forwarded emails subject will contain this prefix */
    pub subject_prefix: String,
    /** S3 bucket to store raw SES emails */
    pub email_bucket: String,
    /** S3 key prefix where SES stores emails */
    pub email_key_prefix: String,
    /**  Enables support for plus(+) sign suffixes on email addresses */
    pub allow_plus: bool,
    /** email_mapping: Dictionary mapping showing where to forward the emails */
    pub email_mapping: HashMap<&str, &str>
}

impl Default for EmailConfig {
    fn default() -> Self {
        let mut email_map: HashMap<&str, &str> = HashMap::with_capacity(1);
        email_map.insert("@nyah.dev", "nyah@hey.com")

        EmailConfig {
            from_email: String::from("hello@nyah.dev"),
            subject_prefix: String::from(""),
            email_bucket: String::from("ses-emails"),
            email_key_prefix: String::from("nyah/"),
            allow_plus: true,
            email_mapping: email_map,
        }
    }
}
