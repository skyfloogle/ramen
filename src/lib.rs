//! TODO: Document stuff here!

#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]

#[macro_use]
pub(crate) mod util;

#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
pub mod input;

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
