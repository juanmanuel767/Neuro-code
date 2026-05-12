use crate::lexer::Token;
use crate::ast::{Expression, Statement};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn previous(&self) -> Option<&Token> {
        if self.pos > 0 {
            self.tokens.get(self.pos - 1)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Some(&Token::EOF) || self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    fn check(&self, expected: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek() == Some(expected)
        }
    }

    fn match_token(&mut self, expected: Token) -> bool {
        if self.check(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Result<&Token, String> {
        if self.check(&expected) {
            Ok(self.advance().unwrap())
        } else {
            Err(format!("Error: {}", message))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.peek() != Some(&Token::EOF) {
                statements.push(self.declaration()?);
            } else {
                break;
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement, String> {
        if self.match_token(Token::Usar) {
            self.usar_declaration()
        } else if self.match_token(Token::Clase) {
            self.class_declaration()
        } else if self.match_token(Token::Funcion) {
            self.function_declaration()
        } else if self.match_token(Token::Asincrono) {
            self.consume(Token::Funcion, "Se esperaba 'funcion' después de 'asincrono'.")?;
            self.async_function_declaration()
        } else if self.match_token(Token::Exportar) {
            if let Some(Token::Identifier(id)) = self.advance().cloned() {
                Ok(Statement::Export(id))
            } else {
                Err("Se esperaba el nombre de la variable o función después de 'exportar'.".into())
            }
        } else {
            self.statement()
        }
    }

    fn usar_declaration(&mut self) -> Result<Statement, String> {
        let modulo = if let Some(Token::Identifier(m)) = self.peek().cloned() {
            self.advance();
            m
        } else if let Some(Token::TextString(s)) = self.peek().cloned() {
            self.advance();
            s
        } else {
            return Err("Se esperaba el nombre de un módulo o ruta en texto después de 'usar'.".into());
        };

        self.consume(Token::Como, "Se esperaba 'como' después del módulo en 'usar'.")?;
        if let Some(Token::Identifier(alias)) = self.advance().cloned() {
            Ok(Statement::Usar(modulo, alias))
        } else {
            Err("Se esperaba un alias después de 'como'.".into())
        }
    }

    fn class_declaration(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la clase.")?;
            let mut methods = Vec::new();
            while !self.check(&Token::CloseBrace) && !self.is_at_end() {
                if self.match_token(Token::Funcion) {
                    methods.push(self.function_declaration()?);
                } else {
                    return Err("Solo se permiten funciones dentro de un cuerpo de clase.".into());
                }
            }
            self.consume(Token::CloseBrace, "Se esperaba '}' después del cuerpo de la clase.")?;
            Ok(Statement::Class(name, methods))
        } else {
            Err("Se esperaba un nombre para la clase.".into())
        }
    }

    fn function_declaration(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::OpenParen, "Se esperaba '(' después del nombre de la función.")?;
            let mut params = Vec::new();
            if !self.check(&Token::CloseParen) {
                loop {
                    if let Some(Token::Identifier(param_name)) = self.advance().cloned() {
                        params.push(param_name);
                    } else {
                        return Err(format!("Normal Function '{}' Error: Se esperaba nombre de parámetro. Token actual: {:?}", name, self.peek()));
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función.")?;
            let body = self.block()?;
            Ok(Statement::Function(name, params, body))
        } else {
            Err("Se esperaba nombre de función.".into())
        }
    }

    fn async_function_declaration(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::OpenParen, "Se esperaba '(' después del nombre de la función asíncrona.")?;
            let mut params = Vec::new();
            if !self.check(&Token::CloseParen) {
                loop {
                    if let Some(Token::Identifier(param_name)) = self.advance().cloned() {
                        params.push(param_name);
                    } else {
                        return Err(format!("Async Function '{}' Error: Se esperaba nombre de parámetro. Token actual: {:?}", name, self.peek()));
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función asíncrona.")?;
            let body = self.block()?;
            Ok(Statement::AsyncFunction(name, params, body))
        } else {
            Err("Se esperaba nombre de función asíncrona.".into())
        }
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.match_token(Token::Si) {
            self.if_statement()
        } else if self.match_token(Token::Mientras) {
            self.while_statement()
        } else if self.match_token(Token::Para) {
            self.for_statement()
        } else if self.match_token(Token::Retornar) {
            self.return_statement()
        } else if self.match_token(Token::Imprimir) {
            self.print_statement()
        } else if self.match_token(Token::Intentar) {
            self.try_catch_statement()
        } else if self.match_token(Token::Lanzar) {
            self.throw_statement()
        } else if self.match_token(Token::Romper) {
            Ok(Statement::Break)
        } else {
            self.expression_statement_or_assign()
        }
    }

    fn if_statement(&mut self) -> Result<Statement, String> {
        let condition = self.expression()?;
        self.consume(Token::OpenBrace, "Se esperaba '{' después de la condición 'si'.")?;
        let then_branch = self.block()?;
        let mut else_branch = Vec::new();
        
        if self.match_token(Token::Sino) {
            if self.match_token(Token::Si) {
                else_branch.push(self.if_statement()?);
            } else {
                self.consume(Token::OpenBrace, "Se esperaba '{' después de 'sino'.")?;
                else_branch = self.block()?;
            }
        }
        
        Ok(Statement::If(condition, then_branch, else_branch))
    }

    fn while_statement(&mut self) -> Result<Statement, String> {
        let condition = self.expression()?;
        self.consume(Token::OpenBrace, "Se esperaba '{' después de la condición 'mientras'.")?;
        let body = self.block()?;
        Ok(Statement::While(condition, body))
    }

    fn for_statement(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(var_name)) = self.advance().cloned() {
            self.consume(Token::En, "Se esperaba 'en' después de la variable 'para'.")?;
            let iterable = self.expression()?;
            self.consume(Token::OpenBrace, "Se esperaba '{' después del iterable 'para'.")?;
            let body = self.block()?;
            Ok(Statement::For(var_name, iterable, body))
        } else {
            Err("Se esperaba un identificador después de 'para'.".into())
        }
    }

    fn return_statement(&mut self) -> Result<Statement, String> {
        let value = self.expression()?;
        Ok(Statement::Return(value))
    }

    fn print_statement(&mut self) -> Result<Statement, String> {
        self.consume(Token::OpenParen, "Se esperaba '(' después de 'imprimir'.")?;
        let mut args = Vec::new();
        if !self.check(&Token::CloseParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::CloseParen, "Se esperaba ')' después de los argumentos de 'imprimir'.")?;
        Ok(Statement::Expression(Expression::FunctionCall("imprimir".to_string(), args)))
    }

    fn block(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.check(&Token::CloseBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(Token::CloseBrace, "Se esperaba '}' al final del bloque.")?;
        Ok(statements)
    }

    fn try_catch_statement(&mut self) -> Result<Statement, String> {
        self.consume(Token::OpenBrace, "Se esperaba '{' después de 'intentar'.")?;
        let try_block = self.block()?;
        
        self.consume(Token::Capturar, "Se esperaba la palabra 'capturar' después del bloque intentar.")?;
        
        let mut error_var = None;
        if let Some(Token::Identifier(var_name)) = self.peek().cloned() {
            self.advance();
            error_var = Some(var_name);
        }
        
        self.consume(Token::OpenBrace, "Se esperaba '{' después de capturar.")?;
        let catch_block = self.block()?;
        
        Ok(Statement::TryCatch(try_block, error_var, catch_block))
    }

    fn throw_statement(&mut self) -> Result<Statement, String> {
        let value = self.expression()?;
        Ok(Statement::Throw(value))
    }

    fn expression_statement_or_assign(&mut self) -> Result<Statement, String> {
        let expr = self.expression()?;
        if self.match_token(Token::Assign) {
            let value = self.expression()?;
            match expr {
                Expression::Identifier(name) => Ok(Statement::Assign(name, value)),
                Expression::MethodCall(callee, method, args) if args.is_empty() => {
                    Ok(Statement::AssignProperty(*callee, method, value))
                },
                Expression::IndexAccess(callee, index) => {
                    Ok(Statement::AssignIndex(*callee, *index, value))
                },
                _ => Err("Solo se puede asignar a variables, propiedades o índices válidos.".into()),
            }
        } else {
            Ok(Statement::Expression(expr))
        }
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.or_expression()
    }

    fn or_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.and_expression()?;
        while self.match_token(Token::O) {
            let right = self.and_expression()?;
            expr = Expression::LogicalOp(Box::new(expr), "o".to_string(), Box::new(right));
        }
        Ok(expr)
    }

    fn and_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.equality()?;
        while self.match_token(Token::Y) {
            let right = self.equality()?;
            expr = Expression::LogicalOp(Box::new(expr), "y".to_string(), Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.comparison()?;
        
        while self.match_token(Token::Equals) || self.match_token(Token::NotEquals) {
            let operator = match self.previous() {
                Some(Token::Equals) => "==",
                Some(Token::NotEquals) => "!=",
                _ => unreachable!(),
            }.to_string();
            let right = self.comparison()?;
            expr = Expression::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.term()?;
        
        while self.match_token(Token::GreaterThan) || self.match_token(Token::GreaterEqual) || 
              self.match_token(Token::LessThan) || self.match_token(Token::LessEqual) {
            let operator = match self.previous() {
                Some(Token::GreaterThan) => ">",
                Some(Token::GreaterEqual) => ">=",
                Some(Token::LessThan) => "<",
                Some(Token::LessEqual) => "<=",
                _ => unreachable!(),
            }.to_string();
            let right = self.term()?;
            expr = Expression::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, String> {
        let mut expr = self.factor()?;
        
        while self.match_token(Token::Plus) || self.match_token(Token::Minus) {
            let operator = match self.previous() {
                Some(Token::Plus) => "+",
                Some(Token::Minus) => "-",
                _ => unreachable!(),
            }.to_string();
            let right = self.factor()?;
            expr = Expression::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.unary()?;
        
        while self.match_token(Token::Multiply) || self.match_token(Token::Divide) {
            let operator = match self.previous() {
                Some(Token::Multiply) => "*",
                Some(Token::Divide) => "/",
                _ => unreachable!(),
            }.to_string();
            let right = self.unary()?;
            expr = Expression::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, String> {
        if self.match_token(Token::No) || self.match_token(Token::Minus) {
            let operator = match self.previous() {
                Some(Token::No) => "no",
                Some(Token::Minus) => "-",
                _ => unreachable!(),
            }.to_string();
            let right = self.unary()?;
            Ok(Expression::UnaryOp(operator, Box::new(right)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expression, String> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(Token::OpenParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(Token::Dot) {
                if let Some(Token::Identifier(method)) = self.advance().cloned() {
                    if self.match_token(Token::OpenParen) {
                        let mut args = Vec::new();
                        if !self.check(&Token::CloseParen) {
                            loop {
                                args.push(self.expression()?);
                                if !self.match_token(Token::Comma) { break; }
                            }
                        }
                        self.consume(Token::CloseParen, "Se esperaba ')' después de los argumentos del método o propiedad.")?;
                        expr = Expression::MethodCall(Box::new(expr), method, args);
                    } else {
                        // En Python/Rust, un dot sin paréntesis puede ser acceso a propiedad (ej: np.array).
                        expr = Expression::MethodCall(Box::new(expr), method, vec![]); 
                    }
                } else {
                    return Err("Se esperaba un nombre de método/propiedad después de '.'.".into());
                }
            } else if self.match_token(Token::OpenBracket) {
                let index = self.expression()?;
                self.consume(Token::CloseBracket, "Se esperaba ']' después del índice.")?;
                expr = Expression::IndexAccess(Box::new(expr), Box::new(index));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression, String> {
        let mut args = Vec::new();
        if !self.check(&Token::CloseParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::CloseParen, "Se esperaba ')' después de los argumentos.")?;
        
        if let Expression::Identifier(name) = callee {
            Ok(Expression::FunctionCall(name, args))
        } else {
            Err("Solo se pueden llamar identificadores y métodos válidos.".into())
        }
    }

    fn primary(&mut self) -> Result<Expression, String> {
        if self.match_token(Token::Falso) { return Ok(Expression::Boolean(false)); }
        if self.match_token(Token::Verdadero) { return Ok(Expression::Boolean(true)); }
        if self.match_token(Token::Nulo) { return Ok(Expression::Null); }
        if self.match_token(Token::Esto) { return Ok(Expression::Identifier("esto".to_string())); }
        
        if self.match_token(Token::Funcion) {
            self.consume(Token::OpenParen, "Se esperaba '(' después de 'funcion' anónima.")?;
            let mut params = Vec::new();
            if !self.check(&Token::CloseParen) {
                loop {
                    if let Some(Token::Identifier(param_name)) = self.advance().cloned() {
                        params.push(param_name);
                    } else { return Err(format!("Lambda expr Error: Se esperaba nombre de parámetro. Token actual: {:?}", self.peek())); }
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función anónima.")?;
            let body = self.block()?;
            return Ok(Expression::LambdaFunction(params, body));
        }

        if self.match_token(Token::Esperar) {
            let expr = self.expression()?;
            return Ok(Expression::Await(Box::new(expr)));
        }
        
        if self.match_token(Token::Nuevo) {
            if let Some(Token::Identifier(class_name)) = self.advance().cloned() {
                self.consume(Token::OpenParen, "Se esperaba '(' después de la clase.")?;
                let mut args = Vec::new();
                if !self.check(&Token::CloseParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(Token::Comma) { break; }
                    }
                }
                self.consume(Token::CloseParen, "Se esperaba ')' después de argumentos.")?;
                return Ok(Expression::NewInstance(class_name, args));
            } else {
                return Err("Se esperaba nombre de clase al invocar 'nuevo'.".into());
            }
        }
        
        if let Some(Token::Number(n)) = self.peek().cloned() {
            self.advance(); return Ok(Expression::Number(n));
        }
        if let Some(Token::IntNumber(n)) = self.peek().cloned() {
            self.advance(); return Ok(Expression::Int(n));
        }
        if let Some(Token::TextString(s)) = self.peek().cloned() {
            self.advance(); return Ok(Expression::Text(s));
        }
        if let Some(Token::Identifier(id)) = self.peek().cloned() {
            self.advance(); return Ok(Expression::Identifier(id));
        }
        
        if self.match_token(Token::OpenParen) {
            let expr = self.expression()?;
            self.consume(Token::CloseParen, "Se esperaba ')' después de la expresión agrupada.")?;
            return Ok(expr);
        }
        
        if self.match_token(Token::OpenBrace) {
            let mut pairs = Vec::new();
            if !self.check(&Token::CloseBrace) {
                loop {
                    let key = self.expression()?;
                    self.consume(Token::Colon, "Se esperaba ':' después de la clave en el diccionario.")?;
                    let value = self.expression()?;
                    pairs.push((key, value));
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::CloseBrace, "Se esperaba '}' al cerrar el diccionario.")?;
            return Ok(Expression::Dictionary(pairs));
        }
        
        if self.match_token(Token::OpenBracket) {
            let mut items = Vec::new();
            if !self.check(&Token::CloseBracket) {
                loop {
                    items.push(self.expression()?);
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::CloseBracket, "Se esperaba ']' después de los elementos de la lista.")?;
            return Ok(Expression::List(items));
        }
        
        Err(format!("Se esperaba una expresión válida. Token inesperado: {:?}", self.peek()))
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_assignment() {
        let code = "edad = 25";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        assert_eq!(ast.len(), 1);
        if let Statement::Assign(name, val) = &ast[0] {
            assert_eq!(name, "edad");
            assert_eq!(*val, Expression::Int(25));
        } else { panic!("Wrong statement"); }
    }
}
