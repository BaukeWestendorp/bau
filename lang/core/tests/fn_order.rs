use bau_core::interpreter::value::Value;
use bau_core::Bau;

#[test]
fn should_be_allowed_to_be_below_call() {
    let src = r#"
        fn main() -> int {
            return bar();
        }
        
        fn bar() -> int {
            return 42;
        }
    "#;

    let val = Bau::new().run(&src.into());
    assert_eq!(val, Ok(Some(Value::Int(42))));
}
