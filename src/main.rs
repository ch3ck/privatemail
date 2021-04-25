//! -*- mode: rust; -*-
//!
//! This file is part of privatemail crate.
//! Copyright (c) 2021 Nyah Check
//! See LICENSE for licensing information.
//!
//! privatemail - This service is for forwarding raw from SES.
//!
//!
//! Authors:
//! - Nyah Check <hello@nyah.dev>
#![allow(clippy::style)]
mod lib;
use lambda_runtime::{handler_fn, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let privatemail_handler = handler_fn(lib::privatemail_handler);
    lambda_runtime::run(privatemail_handler).await?;
    Ok(())
}
