//! A tiny demo crate standing in for Ono-Sendai.

/// Greets a user by name.
pub fn greet(name: &str) -> String {
    format!("hello, {name}")
}

/// Counts the words in a line.
pub fn count_words(line: &str) -> usize {
    line.split_whitespace().count()
}

/// Adds up all values, stopping at the first negative one.
pub fn sum_until_negative(values: &[i64]) -> i64 {
    let mut total = 0;
    for v in values {
        if *v < 0 {
            break;
        }
        total += v;
    }
    total
}
