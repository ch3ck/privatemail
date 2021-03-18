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
 * PrivatEmailConfig:
 *  from_email: Forwarded emails will be received from this SES verified email address.
 *              To match all email addresses on a domain, use a key without the name part of the email(`example.com`)
 *  to_email: Recipient email address. Example: jon@doe.example
 *  subject_prefix: Forwarded emails subject will contain this prefix.
 *  email_bucket: S3 bucket to store raw SES emails.
 *  email_key_prefix: S3 key prefix where SES stores emails.
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
            from_email: env::var("FROM_EMAIL").unwrap_or_default(),
            to_email: env::var("TO_EMAIL").unwrap_or_default(),
            subject_prefix: Some(String::from("PrivateMail: ")), // not currently used
            email_bucket: None,
            email_key_prefix: None,
        }
    }

    /// Create a new PrivatEmailConfig struct.PrivatEmailConfig
    /// You can leave the s3 bucket related fields empty since it's not currently being used
    #[allow(dead_code)]
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

/** Test module for PrivatEmailConfig struct */
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_new_privatemail_config() {
        let new_config = PrivatEmailConfig::new(
            String::from("test_from"),
            String::from("test_to"),
            String::from("test_subject"),
        );
        assert_eq!(new_config.from_email.contains("test_from"), true);
        assert_eq!(new_config.to_email.contains("test_to"), true);
        assert_eq!(new_config.subject_prefix.unwrap(), "test_subject");
        assert_eq!(new_config.email_bucket.is_none(), true);
        assert_eq!(new_config.email_key_prefix.is_none(), true);
    }

    #[test]
    fn test_default_privatemail_config() {
        let new_config = PrivatEmailConfig::default();
        assert_eq!(new_config.from_email.contains("nyah.dev"), true);
        assert_eq!(new_config.to_email.contains("nyah@hey.com"), true);
        assert_eq!(new_config.subject_prefix.is_none(), true);
        assert_eq!(new_config.email_bucket.is_none(), true);
        assert_eq!(new_config.email_key_prefix.is_none(), true);
    }

    #[test]
    fn test_new_from_env_privatemail_config() {
        env::set_var("FROM_EMAIL", "test_from");
        env::set_var("TO_EMAIL", "test_to");

        let new_config = PrivatEmailConfig::new_from_env();
        assert_eq!(new_config.from_email.contains("test_from"), true);
        assert_eq!(new_config.to_email.contains("test_to"), true);
        assert_eq!(new_config.subject_prefix.unwrap(), "PrivateMail: ");
        assert_eq!(new_config.email_bucket.is_none(), true);
        assert_eq!(new_config.email_key_prefix.is_none(), true);
    }
}
