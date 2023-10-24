use bau_core::interpreter::value::Value;
use bau_core::Bau;

#[test]
fn main_with_matched_types_should_not_error() {
    let src = r#"
        fn main() -> string {
            return "hello";
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert_eq!(val, Ok(Some(Value::String("hello".to_string()))));
}

#[test]
fn main_with_missing_return_type_should_error() {
    let src = r#"
        fn main() -> int {
            return "hello";
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}
