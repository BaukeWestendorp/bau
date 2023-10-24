use bau_core::interpreter::value::Value;
use bau_core::Bau;

#[test]
fn let_should_have_type_annotation() {
    let src = r#"
        fn main() -> string {
            let foo = "hello";
            return foo;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn matched_types_should_not_error() {
    let src = r#"
        fn main() -> string {
            let string foo = "hello";
            return foo;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert_eq!(val, Ok(Some(Value::String("hello".to_string()))));
}

#[test]
fn mismatched_types_should_error() {
    let src = r#"
        fn main() -> string {
            let int foo = "hello";
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
            let int foo = 1
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}
