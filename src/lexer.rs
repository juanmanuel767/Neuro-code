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
    
    EOF,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        if ch.is_whitespace() { i += 1; continue; }
        
        // Comentarios
        if ch == '/' && i + 1 < chars.len() && chars[i+1] == '/' {
            while i < chars.len() && chars[i] != '\n' { i += 1; }
            continue;
        }
        
        // Operadores de dos caracteres
        if ch == '=' && i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::Equals); i += 2; continue; }
        if ch == '!' && i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::NotEquals); i += 2; continue; }
        if ch == '>' && i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::GreaterEqual); i += 2; continue; }
        if ch == '<' && i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::LessEqual); i += 2; continue; }
        
        // Operadores de un caracter
        match ch {
            '=' => { tokens.push(Token::Assign); i += 1; continue; },
            '>' => { tokens.push(Token::GreaterThan); i += 1; continue; },
            '<' => { tokens.push(Token::LessThan); i += 1; continue; },
            '+' => { tokens.push(Token::Plus); i += 1; continue; },
            '-' => { tokens.push(Token::Minus); i += 1; continue; },
            '*' => { tokens.push(Token::Multiply); i += 1; continue; },
            '/' => { tokens.push(Token::Divide); i += 1; continue; },
            '{' => { tokens.push(Token::OpenBrace); i += 1; continue; },
            '}' => { tokens.push(Token::CloseBrace); i += 1; continue; },
            '(' => { tokens.push(Token::OpenParen); i += 1; continue; },
            ')' => { tokens.push(Token::CloseParen); i += 1; continue; },
            '[' => { tokens.push(Token::OpenBracket); i += 1; continue; },
            ']' => { tokens.push(Token::CloseBracket); i += 1; continue; },
            ',' => { tokens.push(Token::Comma); i += 1; continue; },
            '.' => { tokens.push(Token::Dot); i += 1; continue; },
            ':' => { tokens.push(Token::Colon); i += 1; continue; },
            '"' | '\'' => {
                let quote = ch;
                i += 1;
                let mut val = String::new();
                while i < chars.len() && chars[i] != quote {
                    val.push(chars[i]);
                    i += 1;
                }
                tokens.push(Token::TextString(val));
                if i < chars.len() { i += 1; }
                continue;
            },
            _ => {}
        }
        
        // Palabras e identificadores
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }
            
            match word.as_str() {
                "si" => tokens.push(Token::Si),
                "sino" => tokens.push(Token::Sino),
                "mientras" => tokens.push(Token::Mientras),
                "para" => tokens.push(Token::Para),
                "en" => tokens.push(Token::En),
                "funcion" => tokens.push(Token::Funcion),
                "retornar" => tokens.push(Token::Retornar),
                "usar" => tokens.push(Token::Usar),
                "como" => tokens.push(Token::Como),
                "intentar" => tokens.push(Token::Intentar),
                "capturar" => tokens.push(Token::Capturar),
                "lanzar" => tokens.push(Token::Lanzar),
                "clase" => tokens.push(Token::Clase),
                "nuevo" => tokens.push(Token::Nuevo),
                "esto" => tokens.push(Token::Esto),
                "asincrono" => tokens.push(Token::Asincrono),
                "esperar" => tokens.push(Token::Esperar),
                "exportar" => tokens.push(Token::Exportar),
                "imprimir" => tokens.push(Token::Imprimir),
                "verdadero" => tokens.push(Token::Verdadero),
                "falso" => tokens.push(Token::Falso),
                "nulo" => tokens.push(Token::Nulo),
                "y" => tokens.push(Token::Y),
                "o" => tokens.push(Token::O),
                "no" => tokens.push(Token::No),
                "romper" => tokens.push(Token::Romper),
                _ => tokens.push(Token::Identifier(word))
            }
            continue;
        }
        
        // Números
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_str.push(chars[i]);
                i += 1;
            }
            if num_str.contains('.') {
                if let Ok(num) = num_str.parse::<f64>() {
                    tokens.push(Token::Number(num));
                }
            } else {
                if let Ok(num) = num_str.parse::<i64>() {
                    tokens.push(Token::IntNumber(num));
                }
            }
            continue;
        }
        
        i += 1; // Skip character if unmatched
    }
    
    tokens.push(Token::EOF);
    tokens
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
}
