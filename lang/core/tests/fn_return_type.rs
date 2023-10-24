use bau::Bau;

#[test]
fn function_with_matched_types_should_not_error() {
    let src = r#"
        fn foo() -> string {
            return "hello";
        }

        fn main() -> void {
            foo();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_ok());
}

#[test]
fn function_with_missing_return_type_should_error() {
    let src = r#"
        fn foo() {
            return "hello";
        }

        fn main() -> void {
            foo();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn function_with_mismatched_return_type_should_error() {
    let src = r#"
        fn foo() -> string {
            return 1;
        }

        fn main() -> void {
            foo();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn return_statement_should_end_with_semicolon() {
    let src = r#"
        fn foo() -> int {
            return 1
        }

        fn main() -> void {
            foo();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

// FIXME: function_should_be_able_to_be_defined_after_call
