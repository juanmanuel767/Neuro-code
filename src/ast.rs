#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: String,
    pub type_name: Option<String>,
}

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
    LambdaFunction(Vec<Param>, Option<String>, Vec<Statement>),
    Await(Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(String, Expression),
    AssignTyped(String, String, Expression),
    AssignProperty(Expression, String, Expression),
    AssignIndex(Expression, Expression, Expression),
    If(Expression, Vec<Statement>, Vec<Statement>),
    While(Expression, Vec<Statement>),
    For(String, Expression, Vec<Statement>), // Para (variable) en (iterable) { cuerpo }
    Function(String, Vec<Param>, Option<String>, Vec<Statement>),
    AsyncFunction(String, Vec<Param>, Option<String>, Vec<Statement>),
    Return(Expression),
    Expression(Expression),
    Usar(String, String), // depredactor/usar modulo/ruta como alias
    Export(String), // exportar un identificador de variable/función al módulo superior
    TryCatch(Vec<Statement>, Option<String>, Vec<Statement>),
    Throw(Expression),
    Class(String, Vec<Statement>),
    Break,
    Parallel(Vec<Statement>),
    Task(Box<Statement>),
    Block(Vec<Statement>),
    Reactive(String, Expression),
    ReactObserve(String, Vec<Statement>),
    Api(Vec<Statement>),
    ApiRoute(String, Vec<Statement>),
}
