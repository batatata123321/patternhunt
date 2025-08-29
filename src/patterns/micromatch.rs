// patterns/micromatch.rs
use crate::error::GlobError;

/// Token types for pattern parsing
#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Char(char),
    Escaped(char),
    OpenParen,
    CloseParen,
    Pipe,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Question,
    Star,
    Plus,
    At,
    Exclamation,
    Comma,
    Dot,
    Caret,
    Dollar,
    Minus,
}

/// Tokenizes the input string into a vector of tokens
fn tokenize(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(Token::Escaped(next));
                } else {
                    out.push(Token::Char('\\'));
                }
            }
            '(' => out.push(Token::OpenParen),
            ')' => out.push(Token::CloseParen),
            '|' => out.push(Token::Pipe),
            '[' => out.push(Token::OpenBracket),
            ']' => out.push(Token::CloseBracket),
            '{' => out.push(Token::OpenBrace),
            '}' => out.push(Token::CloseBrace),
            '?' => out.push(Token::Question),
            '*' => out.push(Token::Star),
            '+' => out.push(Token::Plus),
            '@' => out.push(Token::At),
            '!' => out.push(Token::Exclamation),
            ',' => out.push(Token::Comma),
            '.' => out.push(Token::Dot),
            '^' => out.push(Token::Caret),
            '$' => out.push(Token::Dollar),
            '-' => out.push(Token::Minus),
            ch => out.push(Token::Char(ch)),
        }
    }
    out
}

/// Escapes a character for regex if necessary
fn regex_escape_char(c: char) -> String {
    match c {
        '.' | '+' | '?' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
            format!("\\{}", c)
        }
        other => other.to_string(),
    }
}

/// Collects tokens until a balanced pair of start/end tokens is found
fn collect_until_balanced<I>(
    tokens: &mut std::iter::Peekable<I>,
    start: Token,
    end: Token,
) -> Result<Vec<Token>, GlobError>
where
    I: Iterator<Item = Token>,
{
    let mut out = Vec::new();
    let mut depth = 1usize;

    for token in tokens.by_ref() {
        if token == start {
            depth += 1;
        } else if token == end {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        out.push(token);
    }

    if depth != 0 {
        return Err(GlobError::InvalidPattern(
            "unbalanced parentheses in extglob".into(),
        ));
    }
    Ok(out)
}

/// Converts tokens back to a string
fn tokens_to_string(tokens: &[Token]) -> String {
    let mut s = String::new();
    for token in tokens {
        match token {
            Token::Char(c) => s.push(*c),
            Token::Escaped(c) => {
                s.push('\\');
                s.push(*c);
            }
            Token::OpenParen => s.push('('),
            Token::CloseParen => s.push(')'),
            Token::Pipe => s.push('|'),
            Token::OpenBracket => s.push('['),
            Token::CloseBracket => s.push(']'),
            Token::OpenBrace => s.push('{'),
            Token::CloseBrace => s.push('}'),
            Token::Question => s.push('?'),
            Token::Star => s.push('*'),
            Token::Plus => s.push('+'),
            Token::At => s.push('@'),
            Token::Exclamation => s.push('!'),
            Token::Comma => s.push(','),
            Token::Dot => s.push('.'),
            Token::Caret => s.push('^'),
            Token::Dollar => s.push('$'),
            Token::Minus => s.push('-'),
        }
    }
    s
}

/// Processes extglob patterns and converts them to regex
fn process_extglob(tokens: &[Token], operator: &Token) -> Result<String, GlobError> {
    let mut alternatives = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0;

    for token in tokens {
        if *token == Token::OpenParen {
            depth += 1;
        } else if *token == Token::CloseParen {
            depth -= 1;
        }

        if *token == Token::Pipe && depth == 0 {
            alternatives.push(current.clone());
            current.clear();
        } else {
            current.push(*token);
        }
    }

    if !current.is_empty() {
        alternatives.push(current);
    }

    let mut regex_alternatives = Vec::new();
    for alt in alternatives {
        let alt_str = tokens_to_string(&alt);
        let regex_alt = micromatch_to_regex(&alt_str)?;
        let regex_alt = regex_alt
            .trim_start_matches('^')
            .trim_end_matches('$')
            .to_string();
        regex_alternatives.push(regex_alt);
    }

    let alternatives_str = regex_alternatives.join("|");

    match operator {
        Token::Question => Ok(format!("(?:{})?", alternatives_str)),
        Token::Star => Ok(format!("(?:{})*", alternatives_str)),
        Token::Plus => Ok(format!("(?:{})+", alternatives_str)),
        Token::At => Ok(format!("(?:{})", alternatives_str)),
        Token::Exclamation => Ok(format!("(?!(?:{})).*", alternatives_str)),
        _ => Err(GlobError::InvalidPattern("Invalid extglob operator".into())),
    }
}

/// Processes character class patterns and converts them to regex
fn process_character_class(tokens: &[Token]) -> Result<String, GlobError> {
    let mut class = String::new();
    let mut negated = false;
    let mut first_token = true;

    for token in tokens {
        if first_token {
            if let Token::Exclamation = token {
                negated = true;
                first_token = false;
                continue;
            }
            first_token = false;
        }

        match token {
            Token::Char(c) => class.push(*c),
            Token::Escaped(c) => {
                class.push('\\');
                class.push(*c);
            }
            Token::Dot => class.push('.'),
            Token::Minus => class.push('-'),
            _ => class.push_str(&tokens_to_string(&[*token])),
        }
    }

    if negated {
        Ok(format!("[^{}]", class))
    } else {
        Ok(format!("[{}]", class))
    }
}

/// Converts micromatch patterns to regex strings
///
/// This function handles extended glob patterns and converts them
/// to equivalent regex patterns with proper anchoring.
///
/// # Arguments
///
/// * `pat` - Pattern to convert
///
/// # Returns
///
/// `Ok(String)` with regex pattern, or `Err(GlobError)` on failure
pub fn micromatch_to_regex(pat: &str) -> Result<String, GlobError> {
    // Handle raw regex patterns (prefixed with "re:")
    if let Some(rest) = pat.strip_prefix("re:") {
        return Ok(rest.to_string());
    }

    let tokens = tokenize(pat);
    let mut output = String::new();
    let mut tokens_iter = tokens.into_iter().peekable();

    while let Some(token) = tokens_iter.next() {
        match token {
            Token::Question => output.push('.'),
            Token::Star => output.push_str(".*"),
            Token::Plus => output.push_str(".+"),
            Token::At if tokens_iter.peek() == Some(&Token::OpenParen) => {
                tokens_iter.next();
                let inner =
                    collect_until_balanced(&mut tokens_iter, Token::OpenParen, Token::CloseParen)?;
                let processed = process_extglob(&inner, &Token::At)?;
                output.push_str(&processed);
            }
            Token::Exclamation if tokens_iter.peek() == Some(&Token::OpenParen) => {
                tokens_iter.next();
                let inner =
                    collect_until_balanced(&mut tokens_iter, Token::OpenParen, Token::CloseParen)?;
                let processed = process_extglob(&inner, &Token::Exclamation)?;
                output.push_str(&processed);
            }
            Token::OpenParen
                if matches!(
                    tokens_iter.peek(),
                    Some(
                        Token::Question
                            | Token::Star
                            | Token::Plus
                            | Token::At
                            | Token::Exclamation
                    )
                ) =>
            {
                let operator = tokens_iter.next().unwrap();
                if tokens_iter.peek() != Some(&Token::OpenParen) {
                    return Err(GlobError::InvalidPattern(
                        "Expected OpenParen after extglob operator".into(),
                    ));
                }
                tokens_iter.next();
                let inner =
                    collect_until_balanced(&mut tokens_iter, Token::OpenParen, Token::CloseParen)?;
                let processed = process_extglob(&inner, &operator)?;
                output.push_str(&processed);
            }
            Token::OpenBracket => {
                let inner = collect_until_balanced(
                    &mut tokens_iter,
                    Token::OpenBracket,
                    Token::CloseBracket,
                )?;
                let processed = process_character_class(&inner)?;
                output.push_str(&processed);
            }
            Token::OpenBrace => {
                let inner =
                    collect_until_balanced(&mut tokens_iter, Token::OpenBrace, Token::CloseBrace)?;
                let inner_str = tokens_to_string(&inner);
                let alternatives: Vec<&str> = inner_str.split(',').collect();
                let regex_alternatives: Vec<String> = alternatives
                    .iter()
                    .map(|alt| micromatch_to_regex(alt))
                    .collect::<Result<Vec<_>, _>>()?;
                output.push_str("(?:");
                output.push_str(&regex_alternatives.join("|"));
                output.push(')');
            }
            Token::Escaped(c) => output.push_str(&regex_escape_char(c)),
            Token::Char(c) => output.push_str(&regex_escape_char(c)),
            Token::Dot => output.push_str("\\."),
            Token::Caret => output.push_str("\\^"),
            Token::Dollar => output.push_str("\\$"),
            _ => output.push_str(&tokens_to_string(&[token])),
        }
    }

    Ok(format!("^{}$", output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_patterns() {
        assert_eq!(micromatch_to_regex("*.txt").unwrap(), "^.*\\.txt$");
        assert_eq!(micromatch_to_regex("file?.txt").unwrap(), "^file.\\.txt$");
        assert_eq!(
            micromatch_to_regex("file[0-9].txt").unwrap(),
            "^file[0-9]\\.txt$"
        );
    }

    #[test]
    fn test_extglob_patterns() {
        assert_eq!(micromatch_to_regex("@(a|b)").unwrap(), "^(?:a|b)$");
        assert_eq!(micromatch_to_regex("*(a|b)").unwrap(), "^.*(a|b)$");
        assert_eq!(micromatch_to_regex("+(a|b)").unwrap(), "^.+(a|b)$");
        assert_eq!(micromatch_to_regex("?(a|b)").unwrap(), "^.(a|b)$");
    }

    #[test]
    fn test_brace_expansion() {
        assert_eq!(
            micromatch_to_regex("file.{txt,md}").unwrap(),
            "^file\\.(?:^txt$|^md$)$"
        );
    }
}
