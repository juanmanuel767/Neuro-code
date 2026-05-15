use crate::lexer::{SourcePos, Token};
use crate::ast::{Expression, Param, Statement};

pub struct Parser {
    tokens: Vec<Token>,
    positions: Vec<SourcePos>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let positions = vec![SourcePos { line: 0, column: 0 }; tokens.len()];
        Parser { tokens, positions, pos: 0 }
    }

    pub fn new_with_positions(tokens: Vec<Token>, positions: Vec<SourcePos>) -> Self {
        Parser { tokens, positions, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_pos(&self) -> SourcePos {
        self.positions
            .get(self.pos)
            .copied()
            .or_else(|| self.positions.last().copied())
            .unwrap_or(SourcePos { line: 0, column: 0 })
    }

    fn error_at_current(&self, message: &str) -> String {
        let pos = self.current_pos();
        if pos.line == 0 {
            format!("Error: {}", message)
        } else {
            format!("Error en línea {}, columna {}: {}", pos.line, pos.column, message)
        }
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
            Err(self.error_at_current(message))
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
        if self.match_token(Token::Usar) || self.match_token(Token::Importar) {
            self.usar_declaration()
        } else if self.match_token(Token::Clase) {
            self.class_declaration()
        } else if self.match_token(Token::Funcion) {
            self.function_declaration()
        } else if self.match_token(Token::Asincrono) {
            self.consume(Token::Funcion, "Se esperaba 'funcion' después de 'asincrono'.")?;
            self.async_function_declaration()
        } else if self.match_token(Token::Exportar) {
            let stmt = self.declaration()?;
            Ok(Statement::Export(Box::new(stmt)))
        } else {
            self.statement()
        }
    }

    fn usar_declaration(&mut self) -> Result<Statement, String> {
        let modulo = self.parse_module_spec()?;

        self.consume(Token::Como, "Se esperaba 'como' después del módulo en 'depredactor'.")?;
        if let Some(Token::Identifier(alias)) = self.advance().cloned() {
            Ok(Statement::Usar(modulo, alias))
        } else {
            Err("Se esperaba un alias después de 'como'.".into())
        }
    }

    fn parse_module_spec(&mut self) -> Result<String, String> {
        let first = if let Some(Token::Identifier(m)) = self.peek().cloned() {
            self.advance();
            let mut module_name = m;
            while self.match_token(Token::Dot) {
                if let Some(Token::Identifier(part)) = self.advance().cloned() {
                    module_name.push('.');
                    module_name.push_str(&part);
                } else {
                    return Err(self.error_at_current("Se esperaba un nombre después de '.' en el módulo de 'depredactor'."));
                }
            }
            module_name
        } else if let Some(Token::TextString(s)) = self.peek().cloned() {
            self.advance();
            s
        } else {
            return Err("Se esperaba el nombre de un módulo o ruta en texto después de 'depredactor'.".into());
        };

        if self.match_token(Token::Colon) {
            let target = if let Some(Token::Identifier(m)) = self.peek().cloned() {
                self.advance();
                let mut module_name = m;
                while self.match_token(Token::Dot) {
                    if let Some(Token::Identifier(part)) = self.advance().cloned() {
                        module_name.push('.');
                        module_name.push_str(&part);
                    } else {
                        return Err(self.error_at_current("Se esperaba un nombre después de '.' en el módulo de 'depredactor'."));
                    }
                }
                module_name
            } else if let Some(Token::TextString(s)) = self.peek().cloned() {
                self.advance();
                s
            } else {
                return Err(self.error_at_current("Se esperaba un módulo o ruta después del prefijo de 'depredactor'."));
            };

            Ok(format!("{}:{}", first, target))
        } else {
            Ok(first)
        }
    }

    fn class_declaration(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            let mut super_class = None;
            if self.match_token(Token::Hereda) {
                if let Some(Token::Identifier(parent)) = self.advance().cloned() {
                    super_class = Some(parent);
                } else {
                    return Err("Se esperaba el nombre de la superclase después de 'hereda'.".into());
                }
            }
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la clase.")?;
            let mut methods = Vec::new();
            while !self.check(&Token::CloseBrace) && !self.is_at_end() {
                if self.match_token(Token::Funcion) {
                    methods.push(self.function_declaration()?);
                } else if self.match_token(Token::Asincrono) {
                    if self.match_token(Token::Funcion) {
                        if let Statement::Function(name, params, return_type, body) = self.function_declaration()? {
                            methods.push(Statement::AsyncFunction(name, params, return_type, body));
                        } else {
                            return Err("Declaración de función esperada.".into());
                        }
                    } else {
                        return Err("Se esperaba 'funcion' después de 'asincrono'.".into());
                    }
                } else {
                    return Err("Solo se permiten funciones dentro de un cuerpo de clase.".into());
                }
            }
            self.consume(Token::CloseBrace, "Se esperaba '}' al cerrar la clase.")?;
            Ok(Statement::Class(name, super_class, methods))
        } else {
            Err("Se esperaba un nombre para la clase.".into())
        }
    }

    fn function_declaration(&mut self) -> Result<Statement, String> {
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance(); n
        } else if let Some(token) = self.peek().cloned() {
            match token {
                Token::Numero => { self.advance(); "numero".to_string() }
                Token::Texto => { self.advance(); "texto".to_string() }
                Token::Ruta => { self.advance(); "ruta".to_string() }
                Token::Api => { self.advance(); "api".to_string() }
                _ => return Err("Se esperaba nombre de función.".into()),
            }
        } else {
            return Err("Se esperaba nombre de función.".into());
        };
        self.consume(Token::OpenParen, "Se esperaba '(' después del nombre de la función.")?;
            let params = self.parse_params(&format!("Normal Function '{}'", name))?;
            self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
            let return_type = self.parse_optional_return_type()?;
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función.")?;
            let body = self.block()?;
            Ok(Statement::Function(name, params, return_type, body))
    }

    fn async_function_declaration(&mut self) -> Result<Statement, String> {
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance(); n
        } else if let Some(token) = self.peek().cloned() {
            match token {
                Token::Numero => { self.advance(); "numero".to_string() }
                Token::Texto => { self.advance(); "texto".to_string() }
                Token::Ruta => { self.advance(); "ruta".to_string() }
                Token::Api => { self.advance(); "api".to_string() }
                _ => return Err("Se esperaba nombre de función asíncrona.".into()),
            }
        } else {
            return Err("Se esperaba nombre de función asíncrona.".into());
        };
        self.consume(Token::OpenParen, "Se esperaba '(' después del nombre de la función asíncrona.")?;
        let params = self.parse_params(&format!("Async Function '{}'", name))?;
        self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
        let return_type = self.parse_optional_return_type()?;
        self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función asíncrona.")?;
        let body = self.block()?;
        Ok(Statement::AsyncFunction(name, params, return_type, body))
    }

    fn parse_params(&mut self, context: &str) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if !self.check(&Token::CloseParen) {
            loop {
                let param_name = if let Some(Token::Identifier(param_name)) = self.advance().cloned() {
                    param_name
                } else {
                    return Err(format!("{} Error: Se esperaba nombre de parámetro. Token actual: {:?}", context, self.peek()));
                };
                let type_name = if self.match_token(Token::Colon) {
                    if let Some(Token::Identifier(type_name)) = self.advance().cloned() {
                        Some(type_name)
                    } else {
                        return Err(self.error_at_current("Se esperaba un tipo después de ':' en el parámetro."));
                    }
                } else {
                    None
                };
                params.push(Param { name: param_name, type_name });
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }
        Ok(params)
    }

    fn parse_optional_return_type(&mut self) -> Result<Option<String>, String> {
        if self.match_token(Token::Minus) {
            self.consume(Token::GreaterThan, "Se esperaba '>' después de '-' para declarar tipo de retorno.")?;
            if let Some(Token::Identifier(type_name)) = self.advance().cloned() {
                Ok(Some(type_name))
            } else {
                Err(self.error_at_current("Se esperaba un tipo después de '->'."))
            }
        } else {
            Ok(None)
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
        } else if self.match_token(Token::Continuar) {
            Ok(Statement::Continue)
        } else if self.match_token(Token::Paralelo) {
            self.parallel_statement()
        } else if self.match_token(Token::Tarea) {
            Ok(Statement::Task(Box::new(self.statement()?)))
        } else if self.match_token(Token::OpenBrace) {
            Ok(Statement::Block(self.block()?))
        } else if self.check(&Token::Numero) && matches!(self.tokens.get(self.pos + 1), Some(Token::Identifier(_))) {
            self.advance();
            self.typed_init("numero")
        } else if self.check(&Token::Texto) && matches!(self.tokens.get(self.pos + 1), Some(Token::Identifier(_))) {
            self.advance();
            self.typed_init("texto")
        } else if self.match_token(Token::Reactivo) {
            self.reactive_init()
        } else if self.match_token(Token::Cuando) {
            self.when_statement()
        } else if self.match_token(Token::Api) {
            self.api_statement()
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

    fn parallel_statement(&mut self) -> Result<Statement, String> {
        self.consume(Token::OpenBrace, "Se esperaba '{' después de 'paralelo'.")?;
        let body = self.block()?;
        Ok(Statement::Parallel(body))
    }

    fn typed_init(&mut self, type_name: &str) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::Assign, "Se esperaba '=' en la declaración de variable.")?;
            let value = self.expression()?;
            Ok(Statement::AssignTyped(name, type_name.to_string(), value))
        } else {
            Err("Se esperaba un nombre de variable.".into())
        }
    }

    fn reactive_init(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::Assign, "Se esperaba '=' en la declaración reactiva.")?;
            let value = self.expression()?;
            Ok(Statement::Reactive(name, value))
        } else {
            Err("Se esperaba un nombre para la variable reactiva.".into())
        }
    }

    fn when_statement(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.advance().cloned() {
            self.consume(Token::Cambie, "Se esperaba 'cambie' después del nombre en 'cuando'.")?;
            self.consume(Token::OpenBrace, "Se esperaba '{' para el bloque reactivo.")?;
            let body = self.block()?;
            Ok(Statement::ReactObserve(name, body))
        } else {
            Err("Se esperaba un nombre de variable reactiva.".into())
        }
    }

    fn api_statement(&mut self) -> Result<Statement, String> {
        self.consume(Token::OpenBrace, "Se esperaba '{' después de 'api'.")?;
        let mut routes = Vec::new();
        while !self.check(&Token::CloseBrace) && !self.is_at_end() {
            if self.match_token(Token::Ruta) {
                if let Some(Token::TextString(path)) = self.advance().cloned() {
                    self.consume(Token::OpenBrace, "Se esperaba '{' para el bloque de ruta.")?;
                    let body = self.block()?;
                    routes.push(Statement::ApiRoute(path, body));
                } else {
                    return Err("Se esperaba una ruta en texto.".into());
                }
            } else {
                return Err("Dentro de 'api' solo se permiten bloques de 'ruta'.".into());
            }
        }
        self.consume(Token::CloseBrace, "Se esperaba '}' al cerrar 'api'.")?;
        Ok(Statement::Api(routes))
    }

    fn expression_statement_or_assign(&mut self) -> Result<Statement, String> {
        let mut is_typed_prefix = false;
        let mut name_prefix = String::new();
        
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(n) => { name_prefix = n; is_typed_prefix = true; }
                Token::Numero => { name_prefix = "numero".to_string(); is_typed_prefix = true; }
                Token::Texto => { name_prefix = "texto".to_string(); is_typed_prefix = true; }
                Token::Ruta => { name_prefix = "ruta".to_string(); is_typed_prefix = true; }
                _ => {}
            }
        }

        if is_typed_prefix && self.tokens.get(self.pos + 1) == Some(&Token::Colon) {
            self.advance(); // consume name
            self.advance(); // consume colon
            let type_name = if let Some(Token::Identifier(type_name)) = self.advance().cloned() {
                type_name
            } else {
                return Err(self.error_at_current("Se esperaba un tipo después de ':' en la asignación tipada."));
            };
            self.consume(Token::Assign, "Se esperaba '=' después del tipo en la asignación tipada.")?;
            let value = self.expression()?;
            return Ok(Statement::AssignTyped(name_prefix, type_name, value));
        }

        let expr = self.expression()?;
        
        let assign_token = if self.match_token(Token::Assign) { Some(Token::Assign) }
        else if self.match_token(Token::PlusAssign) { Some(Token::PlusAssign) }
        else if self.match_token(Token::MinusAssign) { Some(Token::MinusAssign) }
        else if self.match_token(Token::MultiplyAssign) { Some(Token::MultiplyAssign) }
        else if self.match_token(Token::DivideAssign) { Some(Token::DivideAssign) }
        else { None };

        if let Some(token) = assign_token {
            let mut value = self.expression()?;
            
            if token != Token::Assign {
                let op = match token {
                    Token::PlusAssign => "+",
                    Token::MinusAssign => "-",
                    Token::MultiplyAssign => "*",
                    Token::DivideAssign => "/",
                    _ => unreachable!(),
                }.to_string();
                value = Expression::BinaryOp(Box::new(expr.clone()), op, Box::new(value));
            }
            
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
        
        while self.match_token(Token::Multiply) || self.match_token(Token::Divide) || self.match_token(Token::Modulo) {
            let operator = match self.previous() {
                Some(Token::Multiply) => "*",
                Some(Token::Divide) => "/",
                Some(Token::Modulo) => "%",
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
                let method = if let Some(Token::Identifier(name)) = self.peek().cloned() {
                    self.advance();
                    name
                } else if let Some(token) = self.peek().cloned() {
                    match token {
                        Token::Si => { self.advance(); "si".to_string() }
                        Token::Sino => { self.advance(); "sino".to_string() }
                        Token::Mientras => { self.advance(); "mientras".to_string() }
                        Token::Para => { self.advance(); "para".to_string() }
                        Token::En => { self.advance(); "en".to_string() }
                        Token::Funcion => { self.advance(); "funcion".to_string() }
                        Token::Retornar => { self.advance(); "retornar".to_string() }
                        Token::Usar => { self.advance(); "usar".to_string() }
                        Token::Como => { self.advance(); "como".to_string() }
                        Token::Intentar => { self.advance(); "intentar".to_string() }
                        Token::Capturar => { self.advance(); "capturar".to_string() }
                        Token::Lanzar => { self.advance(); "lanzar".to_string() }
                        Token::Clase => { self.advance(); "clase".to_string() }
                        Token::Nuevo => { self.advance(); "nuevo".to_string() }
                        Token::Esto => { self.advance(); "esto".to_string() }
                        Token::Asincrono => { self.advance(); "asincrono".to_string() }
                        Token::Esperar => { self.advance(); "esperar".to_string() }
                        Token::Exportar => { self.advance(); "exportar".to_string() }
                        Token::Imprimir => { self.advance(); "imprimir".to_string() }
                        Token::Verdadero => { self.advance(); "verdadero".to_string() }
                        Token::Falso => { self.advance(); "falso".to_string() }
                        Token::Nulo => { self.advance(); "nulo".to_string() }
                        Token::Y => { self.advance(); "y".to_string() }
                        Token::O => { self.advance(); "o".to_string() }
                        Token::No => { self.advance(); "no".to_string() }
                        Token::Romper => { self.advance(); "romper".to_string() }
                        Token::Continuar => { self.advance(); "continuar".to_string() }
                        Token::Paralelo => { self.advance(); "paralelo".to_string() }
                        Token::Tarea => { self.advance(); "tarea".to_string() }
                        Token::Reactivo => { self.advance(); "reactivo".to_string() }
                        Token::Cuando => { self.advance(); "cuando".to_string() }
                        Token::Cambie => { self.advance(); "cambie".to_string() }
                        Token::Api => { self.advance(); "api".to_string() }
                        Token::Ruta => { self.advance(); "ruta".to_string() }
                        Token::Numero => { self.advance(); "numero".to_string() }
                        Token::Texto => { self.advance(); "texto".to_string() }
                        _ => return Err("Se esperaba un nombre de método/propiedad después de '.'.".into()),
                    }
                } else {
                    return Err("Se esperaba un nombre de método/propiedad después de '.'.".into());
                };

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
                    expr = Expression::MethodCall(Box::new(expr), method, vec![]); 
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
        self.consume(Token::CloseParen, "Falta cerrar paréntesis ')'.")?;
        
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
        
        if self.match_token(Token::Super) {
            if self.match_token(Token::Dot) {
                if let Some(Token::Identifier(method_name)) = self.advance().cloned() {
                    self.consume(Token::OpenParen, "Se esperaba '(' después del método 'super'.")?;
                    let mut args = Vec::new();
                    if !self.check(&Token::CloseParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.match_token(Token::Comma) { break; }
                        }
                    }
                    self.consume(Token::CloseParen, "Se esperaba ')' después de los argumentos de 'super'.")?;
                    return Ok(Expression::SuperCall(method_name, args));
                } else {
                    return Err("Se esperaba nombre de método después de 'super.'.".into());
                }
            } else if self.match_token(Token::OpenParen) {
                let mut args = Vec::new();
                if !self.check(&Token::CloseParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(Token::Comma) { break; }
                    }
                }
                self.consume(Token::CloseParen, "Se esperaba ')' después de 'super(...)'.".into())?;
                return Ok(Expression::SuperConstructor(args));
            } else {
                return Err("Se esperaba '.' o '(' después de 'super'.".into());
            }
        }
        
        if self.match_token(Token::Funcion) {
            self.consume(Token::OpenParen, "Se esperaba '(' después de 'funcion' anónima.")?;
            let params = self.parse_params("Lambda expr")?;
            self.consume(Token::CloseParen, "Se esperaba ')' después de los parámetros.")?;
            let return_type = self.parse_optional_return_type()?;
            self.consume(Token::OpenBrace, "Se esperaba '{' antes del cuerpo de la función anónima.")?;
            let body = self.block()?;
            return Ok(Expression::LambdaFunction(params, return_type, body));
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
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(id) => { self.advance(); return Ok(Expression::Identifier(id)); }
                Token::Numero => { self.advance(); return Ok(Expression::Identifier("numero".to_string())); }
                Token::Texto => { self.advance(); return Ok(Expression::Identifier("texto".to_string())); }
                Token::Ruta => { self.advance(); return Ok(Expression::Identifier("ruta".to_string())); }
                _ => {}
            }
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
        
        if let Some(Token::Illegal(c)) = self.peek().cloned() {
            return Err(self.error_at_current(&format!("Carácter inválido: '{}'", c)));
        }
        
        Err(self.error_at_current(&format!("Se esperaba una expresión válida. Token inesperado: {:?}", self.peek())))
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

pub fn parse_with_positions(tokens: Vec<Token>, positions: Vec<SourcePos>) -> Result<Vec<Statement>, String> {
    let mut parser = Parser::new_with_positions(tokens, positions);
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

    #[test]
    fn test_parse_error_includes_position() {
        let code = "imprimir(\"hola\"";
        let (tokens, positions) = crate::lexer::tokenize_with_positions(code);
        let err = parse_with_positions(tokens, positions).unwrap_err();
        assert!(err.contains("línea 1, columna 16"), "error sin posición útil: {}", err);
    }

    #[test]
    fn test_parse_typed_assignment() {
        let code = "edad: Entero = 20";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![Statement::AssignTyped("edad".to_string(), "Entero".to_string(), Expression::Int(20))]);
    }

    #[test]
    fn test_parse_typed_function_params() {
        let code = "funcion sumar(a: Entero, b: Entero) { retornar a + b }";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        match &ast[0] {
            Statement::Function(name, params, _, _) => {
                assert_eq!(name, "sumar");
                assert_eq!(params[0].name, "a");
                assert_eq!(params[0].type_name.as_deref(), Some("Entero"));
                assert_eq!(params[1].name, "b");
                assert_eq!(params[1].type_name.as_deref(), Some("Entero"));
            },
            other => panic!("Se esperaba función, llegó {:?}", other),
        }
    }

    #[test]
    fn test_parse_typed_function_return() {
        let code = "funcion sumar(a: Entero, b: Entero) -> Entero { retornar a + b }";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        match &ast[0] {
            Statement::Function(name, _, return_type, _) => {
                assert_eq!(name, "sumar");
                assert_eq!(return_type.as_deref(), Some("Entero"));
            },
            other => panic!("Se esperaba función, llegó {:?}", other),
        }
    }

    #[test]
    fn test_parse_depredactor_prefixed_python() {
        let code = "depredactor python:xml.etree.ElementTree como xml";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![Statement::Usar("python:xml.etree.ElementTree".to_string(), "xml".to_string())]);
    }

    #[test]
    fn test_parse_depredactor_prefixed_aquila_path() {
        let code = "depredactor aquila:\"../eval_framework.aq\" como fw";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![Statement::Usar("aquila:../eval_framework.aq".to_string(), "fw".to_string())]);
    }
    #[test]
    fn test_parse_illegal_character() {
        let code = "x = @";
        let (tokens, positions) = crate::lexer::tokenize_with_positions(code);
        let err = parse_with_positions(tokens, positions).unwrap_err();
        assert!(err.contains("Carácter inválido: '@'"), "error mensaje incorrecto: {}", err);
    }

    #[test]
    fn test_parse_arithmetic_precedence() {
        let code = "1 + 2 * 3";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        // Should be 1 + (2 * 3)
        if let Statement::Expression(Expression::BinaryOp(left, op, right)) = &ast[0] {
            assert_eq!(op, "+");
            assert!(matches!(**left, Expression::Int(1)));
            if let Expression::BinaryOp(_l2, op2, _r2) = &**right {
                assert_eq!(op2, "*");
            } else { panic!("Expected * on the right side"); }
        } else { panic!("Expected binary op +"); }

        let code2 = "(1 + 2) * 3";
        let tokens2 = tokenize(code2);
        let ast2 = parse(tokens2).unwrap();
        // Should be (1 + 2) * 3
        if let Statement::Expression(Expression::BinaryOp(left, op, right)) = &ast2[0] {
            assert_eq!(op, "*");
            assert!(matches!(**right, Expression::Int(3)));
            if let Expression::BinaryOp(_l2, op2, _r2) = &**left {
                assert_eq!(op2, "+");
            } else { panic!("Expected + on the left side"); }
        } else { panic!("Expected binary op *"); }
    }

    #[test]
    fn test_parse_if_else() {
        let code = "si x == 10 { imprimir(x) } sino { imprimir(0) }";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        if let Statement::If(cond, then_branch, else_branch) = &ast[0] {
            assert!(matches!(cond, Expression::BinaryOp(_, _, _)));
            assert_eq!(then_branch.len(), 1);
            assert_eq!(else_branch.len(), 1);
        } else { panic!("Expected If statement"); }
    }

    #[test]
    fn test_parse_loops() {
        let code = "mientras verdadero { romper }";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        if let Statement::While(cond, body) = &ast[0] {
            assert!(matches!(cond, Expression::Boolean(true)));
            assert_eq!(body[0], Statement::Break);
        } else { panic!("Expected While statement"); }

        let code2 = "para i en lista { imprimir(i) }";
        let tokens2 = tokenize(code2);
        let ast2 = parse(tokens2).unwrap();
        if let Statement::For(var, iter, _body) = &ast2[0] {
            assert_eq!(var, "i");
            assert!(matches!(iter, Expression::Identifier(_)));
        } else { panic!("Expected For statement"); }
    }

    #[test]
    fn test_parse_function_call() {
        let code = "calcular(1, 2, \"test\")";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        if let Statement::Expression(Expression::FunctionCall(name, args)) = &ast[0] {
            assert_eq!(name, "calcular");
            assert_eq!(args.len(), 3);
            assert!(matches!(args[0], Expression::Int(1)));
        } else { panic!("Expected FunctionCall"); }
    }

    #[test]
    fn test_parse_simple_function() {
        let code = "funcion sumar(a, b) { retornar a + b }";
        let tokens = tokenize(code);
        let ast = parse(tokens).unwrap();
        if let Statement::Function(name, params, _, _) = &ast[0] {
            assert_eq!(name, "sumar");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert!(params[0].type_name.is_none());
        } else { panic!("Expected Function"); }
    }

    #[test]
    fn test_parse_syntax_error_message() {
        let code = "si (x == 10) imprimir(x)"; // Missing brace
        let tokens = tokenize(code);
        let err = parse(tokens).unwrap_err();
        assert!(err.contains("Se esperaba '{'"), "Error message should mention missing brace: {}", err);
    }
}
