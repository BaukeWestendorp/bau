#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}
impl Value {
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_true(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Value::Boolean(false))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Value::Integer(value) => value.to_string(),
            Value::Float(value) => value.to_string(),
            Value::Boolean(value) => value.to_string(),
            Value::String(value) => value.to_string(),
        };
        write!(f, "{}", str)
    }
}
