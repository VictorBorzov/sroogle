use std::iter::Iterator;

#[derive(Debug)]
pub struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub fn build(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop_while<P>(&mut self, predicate: P) -> &'a [char]
    where
        P: Fn(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.trim_left();
        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|c| c.is_numeric()).iter().collect());
        } else if self.content[0].is_alphabetic() {
            return Some(
                self.chop_while(|c| c.is_alphanumeric())
                    .iter()
                    .map(|c| c.to_ascii_uppercase())
                    .collect::<String>(),
            );
        }

        let token = &self.content[0..1];
        self.content = &self.content[1..];
        Some(token.iter().collect())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // fn word_iterator_works() {
    //     let phrase = String::from("  hello urmom69420");
    //     let first_word = String::from("hello");
    //     let second_word = String::from("urmom69420");
    //     let mut lexer = Lexer::build(&phrase.chars().collect::<Vec<_>>());
    //     assert_eq!(
    //         lexer.next().unwrap().iter().collect::<Vec<_>>(),
    //         first_word.chars().collect::<Vec<_>>()
    //     );
    //     assert_eq!(
    //         lexer.next().unwrap().iter().collect(),
    //         Some(&second_word.chars().collect())
    //     );
    // }
}
