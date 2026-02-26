import Foundation

struct TextCleaner {
    private static let fillerPhrases = [
        "you know", "i mean", "sort of", "kind of"
    ]

    private static let fillerWords = [
        "um", "uh", "hmm", "hm", "ah", "er"
    ]

    static func clean(_ text: String) -> String {
        var result = text

        // Remove filler phrases first (multi-word, case-insensitive, word boundaries)
        for phrase in fillerPhrases {
            let pattern = "\\b\(NSRegularExpression.escapedPattern(for: phrase))\\b"
            if let regex = try? NSRegularExpression(pattern: pattern, options: .caseInsensitive) {
                result = regex.stringByReplacingMatches(
                    in: result, range: NSRange(result.startIndex..., in: result), withTemplate: ""
                )
            }
        }

        // Remove single filler words (case-insensitive, word boundaries)
        for word in fillerWords {
            let pattern = "\\b\(NSRegularExpression.escapedPattern(for: word))\\b"
            if let regex = try? NSRegularExpression(pattern: pattern, options: .caseInsensitive) {
                result = regex.stringByReplacingMatches(
                    in: result, range: NSRange(result.startIndex..., in: result), withTemplate: ""
                )
            }
        }

        // Collapse stutter repetitions: "I I I think" → "I think"
        if let stutterRegex = try? NSRegularExpression(pattern: "\\b(\\w+)(\\s+\\1)+\\b", options: .caseInsensitive) {
            result = stutterRegex.stringByReplacingMatches(
                in: result, range: NSRange(result.startIndex..., in: result), withTemplate: "$1"
            )
        }

        // Collapse multiple spaces and trim
        result = result.replacingOccurrences(of: "\\s{2,}", with: " ", options: .regularExpression)
        result = result.trimmingCharacters(in: .whitespaces)

        return result
    }
}
