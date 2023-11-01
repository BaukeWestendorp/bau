use bau::interpreter::value::Value;

#[macro_export]
macro_rules! should_run_and_return_value {
    ($value:expr, $code:literal) => {
        let bau = bau::Bau::new();
        let result = bau.run($code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), $value);
    };
}

#[test]
fn fibonaci() {
    should_run_and_return_value!(
        Some(Value::Integer(144)),
        r#"
        fn main() -> int {
            let int result = fibonacci(10);
            return result;
        }

        fn fibonacci(int n) -> int {
            let int a = 0;
            let int b = 1;
            let int i = 0;
            while i < n {
                let int next = a + b;
                a = b;
                b = next;
                i += 1;
            }

            return a + b;
        }
    "#
    );
}

#[test]
fn factorial() {
    should_run_and_return_value!(
        Some(Value::Integer(120)),
        r#"
        fn main() -> int {
            let int result = factorial(5);
            return result;
        }

        fn factorial(int n) -> int {
            let int result = 1;
            let int i = 1;
            while i <= n {
                result *= i;
                i += 1;
            }

            return result;
        }
    "#
    );
}

#[test]
fn fizzbuzz() {
    should_run_and_return_value!(
        Some(Value::String("FizzBuzz".to_string())),
        r#"
        fn main() -> string {
            let string result = fizzbuzz(15);
            return result;
        }

        fn fizzbuzz(int n) -> string {
            let string result = "";
            let int i = 1;
            while i <= n {
                if i % 3 == 0 {
                    result.append("Fizz");
                }
                if i % 5 == 0 {
                    result.append("Buzz");
                }
                if i % 3 != 0 && i % 5 != 0 {
                    result.append(i.to_string());
                }
                i += 1;
            }

            return result;
        }
    "#
    );
}
