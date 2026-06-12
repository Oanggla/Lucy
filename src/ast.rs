#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Expression),
    FnDecl(String, Vec<String>, Expression),
    While(Expression, Expression),
    Expr(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    BinaryOp(String, Box<Expression>, Box<Expression>),
    Block(Vec<Statement>, Option<Box<Expression>>),
    Call(String, Vec<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
}
