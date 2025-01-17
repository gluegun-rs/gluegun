lazy_static::lazy_static! {
    static ref BLESS: bool = std::env::var("BLESS").is_ok();
}

mod test_definition;
pub use test_definition::Test;

mod idl_test;
pub use idl_test::idl_tests;