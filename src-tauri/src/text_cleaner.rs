use regex::Regex;

/// Clean transcribed text by removing filler words and phrases.
/// Port of TextCleaner.swift:4-48.
pub fn clean(text: &str) -> String {
    let mut result = text.to_string();

    // Remove filler phrases first (multi-word, case-insensitive, word boundaries)
    // Matching TextCleaner.swift:16-23
    let filler_phrases = ["you know", "i mean", "sort of", "kind of"];
    for phrase in &filler_phrases {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(phrase));
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, "").to_string();
        }
    }

    // Remove single filler words (case-insensitive, word boundaries)
    // Matching TextCleaner.swift:26-33
    let filler_words = ["um", "uh", "hmm", "hm", "ah", "er"];
    for word in &filler_words {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(word));
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, "").to_string();
        }
    }

    // Collapse stutter repetitions: "I I I think" -> "I think"
    // Matching TextCleaner.swift:36-39
    // Note: Rust's regex crate doesn't support backreferences,
    // so we implement this manually by checking consecutive words.
    result = collapse_stutters(&result);

    // Collapse multiple spaces and trim
    // Matching TextCleaner.swift:43-44
    if let Ok(spaces_re) = Regex::new(r"\s{2,}") {
        result = spaces_re.replace_all(&result, " ").to_string();
    }
    result = result.trim().to_string();

    result
}

/// Collapse consecutive repeated words: "I I I think" -> "I think"
/// This replaces the backreference regex \b(\w+)(\s+\1)+\b which Rust's regex doesn't support.
fn collapse_stutters(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }

    let mut result = Vec::with_capacity(words.len());
    let mut i = 0;

    while i < words.len() {
        result.push(words[i]);
        // Skip consecutive duplicates (case-insensitive)
        while i + 1 < words.len() && words[i].eq_ignore_ascii_case(words[i + 1]) {
            i += 1;
        }
        i += 1;
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filler_removal() {
        assert_eq!(clean("I um think this is uh good"), "I think this is good");
    }

    #[test]
    fn test_phrase_removal() {
        assert_eq!(
            clean("I mean it's you know really good"),
            "it's really good"
        );
    }

    #[test]
    fn test_stutter_collapse() {
        assert_eq!(clean("I I I think so"), "I think so");
    }

    #[test]
    fn test_combined() {
        assert_eq!(
            clean("Um I I mean it's kind of sort of good you know"),
            "I it's good"
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(clean(""), "");
    }

    #[test]
    fn test_no_fillers() {
        assert_eq!(clean("Hello world"), "Hello world");
    }
}
