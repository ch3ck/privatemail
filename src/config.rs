//! -*- mode: rust; -*-
//!
//! This file is part of privatemail crate.
//! Copyright (c) 2021 Nyah Check
//! See LICENSE for licensing information.
//!
//! Authors:
//! - Nyah Check <hello@nyah.dev>
//! GPG signature verification.

//! Configuration struct for `PrivatEmail`
use serde::Serialize;
use std::env;

/// Config object for `PrivatEmail`.
///
/// Implements [`serde::Deserialize`] and [`serde::Serialize`] and
/// can be composed with other consumer configs.
///  `PrivatEmailConfig`:
///  `from_email`: Original Recipient Email from Verified SES Domain
///  `to_email`: Recipient SES verified email address which receives the forwarded email
///  `black_list`: Black listed email addresses.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(default)]
pub struct PrivatEmailConfig {
    /// Original Recipient Email from Verified SES Domain
    pub from_email: String,

    /// Recipient email address that receives the forwarded SES email
    pub to_email: String,

    /// Black Listed email addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub black_list: Option<Vec<String>>,
}

/// Default configuration for `PrivatEmailConfig`
impl Default for PrivatEmailConfig {
    fn default() -> Self {
        PrivatEmailConfig {
            from_email: String::from("hello@nyah.dev"),
            to_email: String::from("nyah@hey.com"),
            black_list: None,
        }
    }
}

/// Create a new `PrivatEmailConfig` client struct from environment variables.
impl PrivatEmailConfig {
    /// Create new PrivatEmailConfig struct from environment variables.
    pub fn new_from_env() -> Self {
        let b_list = env::var("BLACK_LIST").unwrap_or_default();
        let black_list =
            b_list.split(',').map(|x| x.replace(' ', "")).collect();

        PrivatEmailConfig {
            from_email: env::var("FROM_EMAIL")
                .unwrap_or_else(|_e| panic!("Invalid FROM_EMAIL")),
            to_email: env::var("TO_EMAIL")
                .unwrap_or_else(|_e| panic!("Invalid TO_EMAIL")),
            black_list: Some(black_list),
        }
    }

    /// Create a new `PrivatEmailConfig` struct
    pub fn new<F, T, B>(from_email: F, to_email: T, black_list: B) -> Self
    where
        F: ToString,
        T: ToString,
        B: ToString,
    {
        let b_list_vec = black_list.to_string();
        let b_list: Vec<String> =
            b_list_vec.split(',').map(|x| x.replace(' ', "")).collect();
        PrivatEmailConfig {
            from_email: from_email.to_string(),
            to_email: to_email.to_string(),
            black_list: Some(b_list),
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
            String::from("fake@email.t, second@fake.email"),
        );
        assert!(new_config.from_email.contains("test_from"));
        assert!(new_config.to_email.contains("test_to"));
        assert_eq!(
            new_config.black_list.unwrap(),
            ["fake@email.t", "second@fake.email"]
        );
    }
    #[test]
    fn test_default_privatemail_config() {
        let new_config = PrivatEmailConfig::default();
        assert!(new_config.from_email.contains("hello@nyah.dev"));
        assert!(new_config.to_email.contains("nyah@hey.com"));
        assert!(new_config.black_list.is_none());
    }

    #[test]
    fn test_new_from_env_privatemail_config() {
        env::set_var("FROM_EMAIL", "test_from");
        env::set_var("TO_EMAIL", "test_to");

        let new_config = PrivatEmailConfig::new_from_env();
        assert!(new_config.from_email.contains("test_from"));
        assert!(new_config.to_email.contains("test_to"));
        assert_eq!(new_config.black_list.unwrap(), [""]);
    }
}
