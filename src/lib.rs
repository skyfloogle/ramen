#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]

pub mod platform;
pub mod window;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_ne!(9 + 10, 21);
    }
}
