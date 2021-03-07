#![recursion_limit = "256"]
#![allow(clippy::field_reassign_with_default)]

pub mod configs;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_json
extern crate dropshot
