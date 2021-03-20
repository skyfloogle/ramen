//! TODO: Document stuff here!

#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]

pub mod error;
pub mod event;
pub mod platform;
pub mod window;

pub(crate) mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_ne!(9 + 10, 21);
    }
}
