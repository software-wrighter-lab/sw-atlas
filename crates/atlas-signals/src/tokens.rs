//! The tokenizer, and what counts as a partial match.
//!
//! Both are MB01's, behaviour for behaviour: a word is lowercased with
//! punctuation turned to spaces, stop words carry no signal, and a partial
//! match needs five characters because shorter ones match everything.

/// Words that carry no signal, exactly the mockup's list.
const STOP: [&str; 48] = [
    "a",
    "an",
    "the",
    "of",
    "in",
    "on",
    "at",
    "to",
    "for",
    "is",
    "are",
    "was",
    "were",
    "be",
    "i",
    "me",
    "my",
    "you",
    "your",
    "it",
    "this",
    "that",
    "what",
    "where",
    "which",
    "who",
    "how",
    "why",
    "can",
    "could",
    "would",
    "like",
    "want",
    "see",
    "show",
    "take",
    "go",
    "tell",
    "about",
    "something",
    "some",
    "any",
    "and",
    "or",
    "with",
    "from",
    "do",
    "does",
];

/// Content words of a text: lowercased, punctuation to spaces, stop words
/// dropped.
pub fn tokens(text: &str) -> Vec<String> {
    text.chars()
        .map(|c| {
            let lower = c.to_ascii_lowercase();
            if lower.is_ascii_alphanumeric() {
                lower
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|word| !STOP.contains(word))
        .map(str::to_owned)
        .collect()
}

/// Whether an indexed long token and a query word are a partial match: one
/// contains the other, or they share a prefix or suffix of five or more.
pub fn overlaps(indexed: &str, word: &str) -> bool {
    if indexed.contains(word) || word.contains(indexed) {
        return true;
    }
    (5..word.len()).any(|len| {
        word.is_char_boundary(len)
            && (indexed == &word[..len] || indexed == &word[word.len() - len..])
    })
}
