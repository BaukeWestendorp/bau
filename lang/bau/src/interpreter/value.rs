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

    pub fn add(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Integer(this + other),
            (Value::Float(this), Value::Float(other)) => Value::Float(this + other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn subtract(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Integer(this - other),
            (Value::Float(this), Value::Float(other)) => Value::Float(this - other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn multiply(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Integer(this * other),
            (Value::Float(this), Value::Float(other)) => Value::Float(this * other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn divide(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Integer(this / other),
            (Value::Float(this), Value::Float(other)) => Value::Float(this / other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn modulo(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Integer(this % other),
            (Value::Float(this), Value::Float(other)) => Value::Float(this % other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn equals(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this == other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this == other),
            (Value::String(this), Value::String(other)) => Value::Boolean(this == other),
            (Value::Boolean(this), Value::Boolean(other)) => Value::Boolean(this == other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn not_equals(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this != other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this != other),
            (Value::String(this), Value::String(other)) => Value::Boolean(this != other),
            (Value::Boolean(this), Value::Boolean(other)) => Value::Boolean(this != other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn less_than(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this < other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this < other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn less_than_equals(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this <= other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this <= other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn greater_than(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this > other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this > other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
    }

    pub fn greater_than_equals(&mut self, other: Value) {
        let value = match (self.clone(), other) {
            (Value::Integer(this), Value::Integer(other)) => Value::Boolean(this >= other),
            (Value::Float(this), Value::Float(other)) => Value::Boolean(this >= other),
            _ => panic!("Typechhecker should have checked these"),
        };
        *self = value;
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
