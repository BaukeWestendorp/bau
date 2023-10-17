#[derive(Debug, Clone)]
pub enum AstNode {
    FnDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Box<AstNode>,
    },
    LetDeclaration {
        name: String,
        value: Option<Box<AstNode>>,
    },
    NumberLiteral(f64),
    StringLiteral(String),
    Identifier(String),
    Block(Vec<AstNode>),
    Program(Vec<AstNode>),
}
