//! TODO: Document stuff here!

#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]

#[macro_use]
pub(crate) mod util;

pub mod error;
pub mod event;
pub mod monitor;
pub mod platform;
pub mod window;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_ne!(9 + 10, 21);
    }
}
