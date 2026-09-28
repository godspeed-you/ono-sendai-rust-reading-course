//! Evaluation of parsed words.

pub enum Word {
    Number(i64),
    Name(String),
}

/// Parses one word.
pub fn parse(word: &str) -> Word {
    match word.parse::<i64>() {
        Ok(n) => Word::Number(n),
        Err(_) => Word::Name(word.to_string()),
    }
}

/// Evaluates a word against a lookup table.
pub fn eval(word: &Word, lookup: &dyn Fn(&str) -> Option<i64>) -> Option<i64> {
    match word {
        Word::Number(n) => Some(*n),
        Word::Name(name) => lookup(name),
    }
}
