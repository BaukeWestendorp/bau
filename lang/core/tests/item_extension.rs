use bau_core::interpreter::value::Value;
use bau_core::Bau;

#[test]
fn should_return_value_from_extension_method() {
    let src = r#"
        extend string {
	        fn test() -> int {
		        return 42;
	        }
        }
        
        fn main() -> int {
            let string foo = "hello";
            return foo.test();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_ok());
}

#[test]
fn should_not_allow_duplicate_extension_methods_in_multiple_extensions() {
    let src = r#"
        extend string {
	        fn test() -> int {
		        return 42;
	        }
        }
        
        extend string {
	        fn test() -> int {
		        return 42;
	        }
        }

        fn main() -> int {
            let string foo = "hello";
            return foo.test();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn should_not_allow_duplicate_extension_methods_in_single_extension() {
    let src = r#"
        extend string {
	        fn test() -> int {
		        return 42;
	        }
	        
	        fn test() -> int {
		        return 42;
	        }
        }

        fn main() -> int {
            let string foo = "hello";
            return foo.test();
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert!(val.is_err());
}

#[test]
fn should_be_allowed_to_be_below_call() {
    let src = r#"
        fn main() -> int {
            let string foo = "hello";
            return foo.test();
        }
        
        extend string {
	        fn test() -> int {
		        return 42;
	        }
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert_eq!(val, Ok(Some(Value::Int(42))));
}
