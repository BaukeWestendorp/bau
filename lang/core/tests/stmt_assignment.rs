use bau_core::interpreter::value::Value;
use bau_core::Bau;

#[test]
fn matched_types_should_not_error() {
    let src = r#"
        fn main() -> string {
            let string foo = "hello";
            foo = "world";
            return foo;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert_eq!(val, Ok(Some(Value::String("world".to_string()))));
}

#[test]
fn mismatched_types_should_error() {
    let src = r#"
        fn main() -> string {
            let string foo = "hello";
            foo = 1.0;
            return foo;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn should_end_with_semicolon() {
    let src = r#"
        fn main() -> void {
            let string foo = "hello";
            foo = 1.0
            return foo;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}
