#![recursion_limit = "256"]
#![allow(clippy::field_reassign_with_default)]

pub mod config;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_json;
extern crate dropshot;

/** Test module for privatemail package */
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
