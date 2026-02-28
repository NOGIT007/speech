use regex::Regex;
use std::sync::LazyLock;

struct FillerPatterns {
    phrases: Vec<Regex>,
    words: Vec<Regex>,
    artifacts: Regex,
    spaces: Regex,
}

static FILLER_PATTERNS: LazyLock<FillerPatterns> = LazyLock::new(|| {
    let phrases = ["you know", "i mean", "sort of", "kind of"]
        .iter()
        .map(|p| Regex::new(&format!(r"(?i)\b{}\b", regex::escape(p))).unwrap())
        .collect();

    let words = ["um", "uh", "hmm", "hm", "ah", "er"]
        .iter()
        .map(|w| Regex::new(&format!(r"(?i)\b{}\b", regex::escape(w))).unwrap())
        .collect();

    // Model artifacts: bracketed tokens like [BLANK_AUDIO], (silence), *inaudible*, etc.
    let artifacts = Regex::new(r"(?i)\[BLANK_AUDIO\]|\[blank_audio\]|\(blank audio\)|\[SILENCE\]|\[MUSIC\]|\[APPLAUSE\]|\[LAUGHTER\]|\*inaudible\*|\(inaudible\)").unwrap();

    let spaces = Regex::new(r"\s{2,}").unwrap();

    FillerPatterns {
        phrases,
        words,
        artifacts,
        spaces,
    }
});

/// Clean transcribed text by removing filler words and phrases.
/// Port of TextCleaner.swift:4-48.
pub fn clean(text: &str) -> String {
    let patterns = &*FILLER_PATTERNS;
    let mut result = text.to_string();

    // Remove model artifacts like [BLANK_AUDIO], [SILENCE], etc.
    result = patterns.artifacts.replace_all(&result, "").to_string();

    // Remove filler phrases first (multi-word, case-insensitive, word boundaries)
    for re in &patterns.phrases {
        result = re.replace_all(&result, "").to_string();
    }

    // Remove single filler words (case-insensitive, word boundaries)
    for re in &patterns.words {
        result = re.replace_all(&result, "").to_string();
    }

    // Collapse stutter repetitions: "I I I think" -> "I think"
    result = collapse_stutters(&result);

    // Collapse multiple spaces and trim
    result = patterns.spaces.replace_all(&result, " ").to_string();
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

    #[test]
    fn test_blank_audio_removal() {
        assert_eq!(clean("[BLANK_AUDIO]"), "");
        assert_eq!(clean("[BLANK_AUDIO] Hello"), "Hello");
        assert_eq!(clean("Hello [BLANK_AUDIO] world"), "Hello world");
    }

    #[test]
    fn test_model_artifacts_removal() {
        assert_eq!(clean("[SILENCE]"), "");
        assert_eq!(clean("[MUSIC] Hello"), "Hello");
        assert_eq!(clean("Hello [LAUGHTER] world"), "Hello world");
    }
}
