#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number(f64),
    Int(i64),
    Text(String),
    Boolean(bool),
    Null,
    Identifier(String),
    List(Vec<Expression>),
    Dictionary(Vec<(Expression, Expression)>),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    LogicalOp(Box<Expression>, String, Box<Expression>),
    UnaryOp(String, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    IndexAccess(Box<Expression>, Box<Expression>),
    NewInstance(String, Vec<Expression>),
    LambdaFunction(Vec<String>, Vec<Statement>),
    Await(Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(String, Expression),
    AssignProperty(Expression, String, Expression),
    AssignIndex(Expression, Expression, Expression),
    If(Expression, Vec<Statement>, Vec<Statement>),
    While(Expression, Vec<Statement>),
    For(String, Expression, Vec<Statement>), // Para (variable) en (iterable) { cuerpo }
    Function(String, Vec<String>, Vec<Statement>),
    AsyncFunction(String, Vec<String>, Vec<Statement>),
    Return(Expression),
    Expression(Expression),
    Usar(String, String), // usar modulo/ruta como alias
    Export(String), // exportar un identificador de variable/función al módulo superior
    TryCatch(Vec<Statement>, Option<String>, Vec<Statement>),
    Throw(Expression),
    Class(String, Vec<Statement>),
    Break,
}
