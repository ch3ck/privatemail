//! -*- mode: rust; -*-
//!
//! This file is part of privatemail crate.
//! Copyright (c) 2022 Nyah Check
//! See LICENSE for licensing information.
//!
//! privatemail - This service is for forwarding raw from SES.
//!
//!
//! Authors:
//! - Nyah Check <hello@nyah.dev>

use lambda_runtime::{service_fn, Error};
use lib::privatemail_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let privatemail_handler = service_fn(privatemail_handler);
    lambda_runtime::run(privatemail_handler).await?;
    Ok(())
}
