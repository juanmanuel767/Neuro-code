#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords en español
    Si,
    Sino,
    Mientras,
    Para,
    En,
    Funcion,
    Retornar,
    Usar,
    Como,
    Intentar,
    Capturar,
    Lanzar,
    Clase,
    Nuevo,
    Esto,
    Asincrono,
    Esperar,
    Exportar,
    Imprimir,
    Verdadero,
    Falso,
    Nulo,
    Y,
    O,
    No,
    Romper,
    Paralelo,
    Tarea,
    Reactivo,
    Cuando,
    Cambie,
    Api,
    Ruta,
    Numero,
    Texto,
    
    // Identificadores y Literales
    Identifier(String),
    Number(f64),
    IntNumber(i64),
    TextString(String),
    
    // Símbolos
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Dot,
    Colon,
    
    // Operadores
    Assign,       // =
    Equals,       // ==
    NotEquals,    // !=
    GreaterThan,  // >
    LessThan,     // <
    GreaterEqual, // >=
    LessEqual,    // <=
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    
    Illegal(char),
    EOF,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    tokenize_with_positions(input).0
}

pub fn tokenize_with_positions(input: &str) -> (Vec<Token>, Vec<SourcePos>) {
    let mut tokens = Vec::new();
    let mut positions = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut line = 1usize;
    let mut column = 1usize;

    fn push_token(tokens: &mut Vec<Token>, positions: &mut Vec<SourcePos>, token: Token, line: usize, column: usize) {
        tokens.push(token);
        positions.push(SourcePos { line, column });
    }

    fn advance_char(ch: char, line: &mut usize, column: &mut usize) {
        if ch == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
    
    while i < chars.len() {
        let ch = chars[i];
        
        if ch.is_whitespace() {
            advance_char(ch, &mut line, &mut column);
            i += 1;
            continue;
        }
        
        // Comentarios
        if ch == '/' && i + 1 < chars.len() && chars[i+1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                advance_char(chars[i], &mut line, &mut column);
                i += 1;
            }
            continue;
        }

        let start_line = line;
        let start_column = column;
        
        // Operadores de dos caracteres
        if ch == '=' && i + 1 < chars.len() && chars[i+1] == '=' { push_token(&mut tokens, &mut positions, Token::Equals, start_line, start_column); advance_char(chars[i], &mut line, &mut column); advance_char(chars[i+1], &mut line, &mut column); i += 2; continue; }
        if ch == '!' && i + 1 < chars.len() && chars[i+1] == '=' { push_token(&mut tokens, &mut positions, Token::NotEquals, start_line, start_column); advance_char(chars[i], &mut line, &mut column); advance_char(chars[i+1], &mut line, &mut column); i += 2; continue; }
        if ch == '>' && i + 1 < chars.len() && chars[i+1] == '=' { push_token(&mut tokens, &mut positions, Token::GreaterEqual, start_line, start_column); advance_char(chars[i], &mut line, &mut column); advance_char(chars[i+1], &mut line, &mut column); i += 2; continue; }
        if ch == '<' && i + 1 < chars.len() && chars[i+1] == '=' { push_token(&mut tokens, &mut positions, Token::LessEqual, start_line, start_column); advance_char(chars[i], &mut line, &mut column); advance_char(chars[i+1], &mut line, &mut column); i += 2; continue; }
        
        // Operadores de un caracter
        match ch {
            '=' => { push_token(&mut tokens, &mut positions, Token::Assign, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '>' => { push_token(&mut tokens, &mut positions, Token::GreaterThan, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '<' => { push_token(&mut tokens, &mut positions, Token::LessThan, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '+' => { push_token(&mut tokens, &mut positions, Token::Plus, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '-' => { push_token(&mut tokens, &mut positions, Token::Minus, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '*' => { push_token(&mut tokens, &mut positions, Token::Multiply, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '/' => { push_token(&mut tokens, &mut positions, Token::Divide, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '{' => { push_token(&mut tokens, &mut positions, Token::OpenBrace, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '}' => { push_token(&mut tokens, &mut positions, Token::CloseBrace, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '(' => { push_token(&mut tokens, &mut positions, Token::OpenParen, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            ')' => { push_token(&mut tokens, &mut positions, Token::CloseParen, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '[' => { push_token(&mut tokens, &mut positions, Token::OpenBracket, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            ']' => { push_token(&mut tokens, &mut positions, Token::CloseBracket, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            ',' => { push_token(&mut tokens, &mut positions, Token::Comma, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '.' => { push_token(&mut tokens, &mut positions, Token::Dot, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            ':' => { push_token(&mut tokens, &mut positions, Token::Colon, start_line, start_column); advance_char(ch, &mut line, &mut column); i += 1; continue; },
            '"' | '\'' => {
                let quote = ch;
                advance_char(ch, &mut line, &mut column);
                i += 1;
                let mut val = String::new();
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        let escaped = chars[i + 1];
                        let resolved = match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            '"' => '"',
                            '\'' => '\'',
                            other => other,
                        };
                        val.push(resolved);
                        advance_char(chars[i], &mut line, &mut column);
                        advance_char(chars[i + 1], &mut line, &mut column);
                        i += 2;
                    } else {
                        val.push(chars[i]);
                        advance_char(chars[i], &mut line, &mut column);
                        i += 1;
                    }
                }
                push_token(&mut tokens, &mut positions, Token::TextString(val), start_line, start_column);
                if i < chars.len() {
                    advance_char(chars[i], &mut line, &mut column);
                    i += 1;
                }
                continue;
            },
            _ => {}
        }
        
        // Palabras e identificadores
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                advance_char(chars[i], &mut line, &mut column);
                i += 1;
            }
            
            let token = match word.as_str() {
                "si" => Token::Si,
                "sino" => Token::Sino,
                "mientras" => Token::Mientras,
                "para" => Token::Para,
                "en" => Token::En,
                "funcion" => Token::Funcion,
                "retornar" => Token::Retornar,
                "usar" => Token::Usar,
                "depredactor" => Token::Usar,
                "como" => Token::Como,
                "intentar" => Token::Intentar,
                "capturar" => Token::Capturar,
                "lanzar" => Token::Lanzar,
                "clase" => Token::Clase,
                "nuevo" => Token::Nuevo,
                "esto" => Token::Esto,
                "asincrono" => Token::Asincrono,
                "esperar" => Token::Esperar,
                "exportar" => Token::Exportar,
                "imprimir" => Token::Imprimir,
                "verdadero" => Token::Verdadero,
                "falso" => Token::Falso,
                "nulo" => Token::Nulo,
                "y" => Token::Y,
                "o" => Token::O,
                "no" => Token::No,
                "romper" => Token::Romper,
                "paralelo" => Token::Paralelo,
                "tarea" => Token::Tarea,
                "reactivo" => Token::Reactivo,
                "cuando" => Token::Cuando,
                "cambie" => Token::Cambie,
                "api" => Token::Api,
                "ruta" => Token::Ruta,
                "numero" => Token::Numero,
                "texto" => Token::Texto,
                _ => Token::Identifier(word)
            };
            push_token(&mut tokens, &mut positions, token, start_line, start_column);
            continue;
        }
        
        // Números
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_str.push(chars[i]);
                advance_char(chars[i], &mut line, &mut column);
                i += 1;
            }
            if num_str.contains('.') {
                if let Ok(num) = num_str.parse::<f64>() {
                    push_token(&mut tokens, &mut positions, Token::Number(num), start_line, start_column);
                }
            } else {
                if let Ok(num) = num_str.parse::<i64>() {
                    push_token(&mut tokens, &mut positions, Token::IntNumber(num), start_line, start_column);
                }
            }
            continue;
        }
        
        push_token(&mut tokens, &mut positions, Token::Illegal(ch), start_line, start_column);
        advance_char(ch, &mut line, &mut column);
        i += 1;
    }
    
    push_token(&mut tokens, &mut positions, Token::EOF, line, column);
    (tokens, positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords_and_vars() {
        let tokens = tokenize("usar numpy como np\nedad = 25");
        assert_eq!(tokens[0], Token::Usar);
        assert_eq!(tokens[1], Token::Identifier("numpy".to_string()));
        assert_eq!(tokens[2], Token::Como);
        assert_eq!(tokens[3], Token::Identifier("np".to_string()));
        assert_eq!(tokens[4], Token::Identifier("edad".to_string()));
        assert_eq!(tokens[5], Token::Assign);
        assert_eq!(tokens[6], Token::IntNumber(25));
    }

    #[test]
    fn test_depredactor_alias() {
        let tokens = tokenize("depredactor math como m");
        assert_eq!(tokens[0], Token::Usar);
        assert_eq!(tokens[1], Token::Identifier("math".to_string()));
        assert_eq!(tokens[2], Token::Como);
        assert_eq!(tokens[3], Token::Identifier("m".to_string()));
    }

    #[test]
    fn test_token_positions() {
        let (tokens, positions) = tokenize_with_positions("imprimir(\"ok\")\nedad = 25");
        assert_eq!(tokens[0], Token::Imprimir);
        assert_eq!(positions[0], SourcePos { line: 1, column: 1 });
        assert_eq!(tokens[4], Token::Identifier("edad".to_string()));
        assert_eq!(positions[4], SourcePos { line: 2, column: 1 });
    }

    #[test]
    fn test_string_escapes() {
        let tokens = tokenize(r#"json = "{\"nombre\":\"Aquila\"}"
texto = 'linea\nnueva\tok'"#);
        assert_eq!(tokens[2], Token::TextString("{\"nombre\":\"Aquila\"}".to_string()));
        assert_eq!(tokens[5], Token::TextString("linea\nnueva\tok".to_string()));
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        let tokens = tokenize("\"\"");
        assert_eq!(tokens[0], Token::TextString("".to_string()));

        // Negative numbers (as operators + numbers)
        let tokens = tokenize("-123 -45.67");
        assert_eq!(tokens[0], Token::Minus);
        assert_eq!(tokens[1], Token::IntNumber(123));
        assert_eq!(tokens[2], Token::Minus);
        assert_eq!(tokens[3], Token::Number(45.67));

        // Invalid characters
        let tokens = tokenize("@#$");
        assert_eq!(tokens[0], Token::Illegal('@'));
        assert_eq!(tokens[1], Token::Illegal('#'));
        assert_eq!(tokens[2], Token::Illegal('$'));
    }

    #[test]
    fn test_all_token_types() {
        let code = "si sino mientras para en funcion retornar usar como intentar capturar lanzar clase nuevo esto asincrono esperar exportar imprimir verdadero falso nulo y o no romper paralelo tarea reactivo cuando cambie api ruta numero texto var_123 100 3.14 \"cadena\" + - * / = == != > < >= <= ( ) { } [ ] , . :";
        let tokens = tokenize(code);
        
        let expected = vec![
            Token::Si, Token::Sino, Token::Mientras, Token::Para, Token::En, Token::Funcion, 
            Token::Retornar, Token::Usar, Token::Como, Token::Intentar, Token::Capturar, 
            Token::Lanzar, Token::Clase, Token::Nuevo, Token::Esto, Token::Asincrono, 
            Token::Esperar, Token::Exportar, Token::Imprimir, Token::Verdadero, Token::Falso, 
            Token::Nulo, Token::Y, Token::O, Token::No, Token::Romper, Token::Paralelo, 
            Token::Tarea, Token::Reactivo, Token::Cuando, Token::Cambie, Token::Api, 
            Token::Ruta, Token::Numero, Token::Texto,
            Token::Identifier("var_123".to_string()),
            Token::IntNumber(100),
            Token::Number(3.14),
            Token::TextString("cadena".to_string()),
            Token::Plus, Token::Minus, Token::Multiply, Token::Divide, Token::Assign,
            Token::Equals, Token::NotEquals, Token::GreaterThan, Token::LessThan,
            Token::GreaterEqual, Token::LessEqual,
            Token::OpenParen, Token::CloseParen,
            Token::OpenBrace, Token::CloseBrace,
            Token::OpenBracket, Token::CloseBracket,
            Token::Comma, Token::Dot, Token::Colon,
            Token::EOF
        ];
        
        for (i, t) in expected.iter().enumerate() {
            assert_eq!(tokens[i], *t, "Mismatch at index {}", i);
        }
    }
}
