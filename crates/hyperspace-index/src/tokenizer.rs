//! Tokenizer pipeline for BM25 Turbo.
//!
//! Pipeline stages: regex splitting -> lowercase -> stopword removal -> stemming -> vocabulary mapping.
//! Each stage is optional and configurable. Accepts a custom `Fn(&str) -> Vec<String>`
//! for user-provided tokenization.

use std::collections::HashMap;

use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

use crate::stopwords;

/// Custom tokenizer function type.
type TokenizerFn = Box<dyn Fn(&str) -> Vec<String> + Send + Sync>;

/// Configurable text tokenizer.
pub struct Tokenizer {
    /// Compiled regex for token splitting.
    pattern: Regex,
    /// Whether to lowercase tokens.
    lowercase: bool,
    /// Optional stopword set.
    stopwords: Option<std::collections::HashSet<String>>,
    /// Optional stemmer.
    stemmer: Option<Stemmer>,
    /// Optional custom tokenizer function (overrides regex + lowercase + stopwords + stemming).
    custom_fn: Option<TokenizerFn>,
}

impl std::fmt::Debug for Tokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokenizer")
            .field("pattern", &self.pattern.as_str())
            .field("lowercase", &self.lowercase)
            .field("has_stopwords", &self.stopwords.is_some())
            .field("has_stemmer", &self.stemmer.is_some())
            .field("has_custom_fn", &self.custom_fn.is_some())
            .finish()
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        // The default regex r"\w+" is always valid, so this cannot fail.
        // Using expect here is safe because the pattern is a compile-time constant.
        Self::builder()
            .build()
            .expect("default tokenizer regex r\"\\w+\" is always valid")
    }
}

/// Builder for constructing a [`Tokenizer`].
pub struct TokenizerBuilder {
    pattern: String,
    lowercase: bool,
    stopwords: Option<Vec<String>>,
    stemmer_algorithm: Option<Algorithm>,
    custom_fn: Option<TokenizerFn>,
}

impl Default for TokenizerBuilder {
    fn default() -> Self {
        Self {
            pattern: r"\w+".to_string(),
            lowercase: true,
            stopwords: None,
            stemmer_algorithm: None,
            custom_fn: None,
        }
    }
}

impl TokenizerBuilder {
    /// Set the regex pattern for splitting text into tokens.
    #[must_use]
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = pattern.to_string();
        self
    }

    /// Enable or disable lowercasing.
    #[must_use]
    pub fn lowercase(mut self, yes: bool) -> Self {
        self.lowercase = yes;
        self
    }

    /// Set stopwords to filter out.
    #[must_use]
    pub fn stopwords(mut self, words: Vec<String>) -> Self {
        self.stopwords = Some(words);
        self
    }

    /// Set the stemming algorithm.
    ///
    /// Supports all 17 Snowball languages via [`rust_stemmers::Algorithm`]:
    /// Arabic, Danish, Dutch, English, Finnish, French, German, Hungarian,
    /// Italian, Norwegian, Portuguese, Romanian, Russian, Spanish, Swedish,
    /// Tamil, Turkish.
    #[must_use]
    pub fn stemmer(mut self, algorithm: Algorithm) -> Self {
        self.stemmer_algorithm = Some(algorithm);
        self
    }

    /// Set a custom tokenizer function that replaces the entire pipeline.
    #[must_use]
    pub fn custom_fn(mut self, f: impl Fn(&str) -> Vec<String> + Send + Sync + 'static) -> Self {
        self.custom_fn = Some(Box::new(f));
        self
    }

    /// Configure the tokenizer for a specific language.
    ///
    /// Sets both the stemmer algorithm and the stopword list for the
    /// given language. This is a convenience method combining
    /// [`stemmer`](Self::stemmer) and [`stopwords`](Self::stopwords).
    ///
    /// Supported languages: Arabic, Danish, Dutch, English, Finnish, French,
    /// German, Hungarian, Italian, Norwegian, Portuguese, Romanian, Russian,
    /// Spanish, Swedish, Turkish, Hindi (stopwords only, no stemmer).
    ///
    /// Returns the builder unchanged if the language is not recognized.
    #[must_use]
    pub fn language(mut self, language: &str) -> Self {
        if let Some(words) = stopwords::for_language(language) {
            self.stopwords = Some(words);
        }

        let algorithm = match language.to_lowercase().as_str() {
            "arabic" | "ar" => Some(Algorithm::Arabic),
            "danish" | "da" => Some(Algorithm::Danish),
            "dutch" | "nl" => Some(Algorithm::Dutch),
            "english" | "en" => Some(Algorithm::English),
            "finnish" | "fi" => Some(Algorithm::Finnish),
            "french" | "fr" => Some(Algorithm::French),
            "german" | "de" => Some(Algorithm::German),
            "hungarian" | "hu" => Some(Algorithm::Hungarian),
            "italian" | "it" => Some(Algorithm::Italian),
            "norwegian" | "no" => Some(Algorithm::Norwegian),
            "portuguese" | "pt" => Some(Algorithm::Portuguese),
            "romanian" | "ro" => Some(Algorithm::Romanian),
            "russian" | "ru" => Some(Algorithm::Russian),
            "spanish" | "es" => Some(Algorithm::Spanish),
            "swedish" | "sv" => Some(Algorithm::Swedish),
            "turkish" | "tr" => Some(Algorithm::Turkish),
            // Tamil has a stemmer but no stopword list in our set
            "tamil" | "ta" => Some(Algorithm::Tamil),
            // Hindi has stopwords but no Snowball stemmer
            _ => None,
        };

        if let Some(algo) = algorithm {
            self.stemmer_algorithm = Some(algo);
        }

        self
    }

    /// Build the tokenizer.
    pub fn build(self) -> Result<Tokenizer, String> {
        let pattern = Regex::new(&self.pattern).map_err(|e| e.to_string())?;

        let stopwords = self
            .stopwords
            .map(|words| words.into_iter().collect::<std::collections::HashSet<_>>());

        let stemmer = self.stemmer_algorithm.map(Stemmer::create);

        Ok(Tokenizer {
            pattern,
            lowercase: self.lowercase,
            stopwords,
            stemmer,
            custom_fn: self.custom_fn,
        })
    }
}

impl Tokenizer {
    /// Create a new [`TokenizerBuilder`].
    pub fn builder() -> TokenizerBuilder {
        TokenizerBuilder::default()
    }

    /// Tokenize a text string into a list of token strings.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        if let Some(ref f) = self.custom_fn {
            return f(text);
        }

        self.pattern
            .find_iter(text)
            .map(|m| {
                let mut token = m.as_str().to_string();
                if self.lowercase {
                    token = token.to_lowercase();
                }
                if let Some(ref stemmer) = self.stemmer {
                    token = stemmer.stem(&token).to_string();
                }
                token
            })
            .filter(|token| {
                if let Some(ref sw) = self.stopwords {
                    !sw.contains(token)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Tokenize and map tokens to vocabulary IDs, updating the vocabulary
    /// if new tokens are encountered.
    pub fn tokenize_with_vocab(
        &self,
        text: &str,
        vocab: &mut HashMap<String, u32>,
    ) -> (Vec<u32>, u32) {
        let tokens = self.tokenize(text);
        let length = tokens.len() as u32;
        let token_ids = tokens
            .into_iter()
            .map(|t| {
                let next_id = vocab.len() as u32;
                *vocab.entry(t).or_insert(next_id)
            })
            .collect();

        (token_ids, length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tokenizer_splits_words() {
        let tok = Tokenizer::default();
        let tokens = tok.tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn custom_fn_overrides_pipeline() {
        let tok = Tokenizer::builder()
            .custom_fn(|s| s.split('-').map(String::from).collect())
            .build()
            .unwrap();
        let tokens = tok.tokenize("a-b-c");
        assert_eq!(tokens, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let tok = Tokenizer::default();
        let tokens = tok.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn tokenize_with_vocab_empty_input() {
        let tok = Tokenizer::default();
        let mut vocab = HashMap::new();
        let result = tok.tokenize_with_vocab("", &mut vocab);
        assert!(result.0.is_empty());
        assert_eq!(result.1, 0);
    }

    #[test]
    fn language_english_sets_stemmer_and_stopwords() {
        let tok = Tokenizer::builder().language("english").build().unwrap();
        // "the" is a stopword, "running" should be stemmed to "run"
        let tokens = tok.tokenize("the running fox");
        assert!(
            !tokens.contains(&"the".to_string()),
            "stopword 'the' should be removed"
        );
        assert!(
            tokens.contains(&"run".to_string()),
            "running should be stemmed to run"
        );
    }

    #[test]
    fn language_german_sets_stemmer_and_stopwords() {
        let tok = Tokenizer::builder().language("german").build().unwrap();
        let tokens = tok.tokenize("der schnelle Hund");
        assert!(
            !tokens.contains(&"der".to_string()),
            "German stopword 'der' should be removed"
        );
    }

    #[test]
    fn language_french_sets_stemmer_and_stopwords() {
        let tok = Tokenizer::builder().language("french").build().unwrap();
        let tokens = tok.tokenize("le chat rapide");
        assert!(
            !tokens.contains(&"le".to_string()),
            "French stopword 'le' should be removed"
        );
    }

    #[test]
    fn stemmer_all_17_algorithms() {
        // Verify that all Algorithm variants can be set and build
        let algorithms = [
            Algorithm::Arabic,
            Algorithm::Danish,
            Algorithm::Dutch,
            Algorithm::English,
            Algorithm::Finnish,
            Algorithm::French,
            Algorithm::German,
            Algorithm::Hungarian,
            Algorithm::Italian,
            Algorithm::Norwegian,
            Algorithm::Portuguese,
            Algorithm::Romanian,
            Algorithm::Russian,
            Algorithm::Spanish,
            Algorithm::Swedish,
            Algorithm::Tamil,
            Algorithm::Turkish,
        ];

        for algo in &algorithms {
            let tok = Tokenizer::builder().stemmer(*algo).build();
            assert!(
                tok.is_ok(),
                "Failed to build tokenizer with stemmer {algo:?}"
            );
        }
    }

    #[test]
    fn stemmer_english_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::English)
            .build()
            .unwrap();
        let tokens = tok.tokenize("running jumps easily");
        assert!(tokens.contains(&"run".to_string()));
        assert!(tokens.contains(&"jump".to_string()));
        assert!(tokens.contains(&"easili".to_string()));
    }

    #[test]
    fn stemmer_french_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::French)
            .build()
            .unwrap();
        // "maisons" -> "maison"
        let tokens = tok.tokenize("maisons");
        assert!(tokens.contains(&"maison".to_string()));
    }

    #[test]
    fn stemmer_german_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::German)
            .build()
            .unwrap();
        // "Häuser" -> lowercased "häuser" -> stemmed
        let tokens = tok.tokenize("Katzen");
        assert!(!tokens.is_empty(), "German stemmer should produce tokens");
    }

    #[test]
    fn language_convenience_all_supported() {
        let langs = [
            "english",
            "german",
            "french",
            "spanish",
            "italian",
            "portuguese",
            "dutch",
            "russian",
            "swedish",
            "norwegian",
            "danish",
            "finnish",
            "hungarian",
            "romanian",
            "turkish",
            "arabic",
        ];
        for lang in &langs {
            let tok = Tokenizer::builder().language(lang).build();
            assert!(tok.is_ok(), "Failed to build tokenizer for language {lang}");
        }
    }

    // ---------------------------------------------------------------
    // TEST-P1-005: Stemmer Language Support -- known word -> stem mappings
    // ---------------------------------------------------------------

    #[test]
    fn stemmer_spanish_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Spanish)
            .build()
            .unwrap();
        // "corriendo" (running) -> stem
        let tokens = tok.tokenize("corriendo");
        assert!(!tokens.is_empty(), "Spanish stemmer should produce tokens");
    }

    #[test]
    fn stemmer_italian_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Italian)
            .build()
            .unwrap();
        // "gatti" (cats) -> "gatt"
        let tokens = tok.tokenize("gatti");
        assert!(
            tokens.contains(&"gatt".to_string()),
            "Italian: 'gatti' should stem to 'gatt', got {tokens:?}"
        );
    }

    #[test]
    fn stemmer_portuguese_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Portuguese)
            .build()
            .unwrap();
        let tokens = tok.tokenize("correndo");
        assert!(
            !tokens.is_empty(),
            "Portuguese stemmer should produce tokens"
        );
    }

    #[test]
    fn stemmer_dutch_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Dutch)
            .build()
            .unwrap();
        let tokens = tok.tokenize("katten");
        assert!(!tokens.is_empty(), "Dutch stemmer should produce tokens");
    }

    #[test]
    fn stemmer_russian_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Russian)
            .build()
            .unwrap();
        let tokens = tok.tokenize("\u{0434}\u{043e}\u{043c}\u{0430}"); // "дома"
        assert!(!tokens.is_empty(), "Russian stemmer should produce tokens");
    }

    #[test]
    fn stemmer_swedish_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Swedish)
            .build()
            .unwrap();
        let tokens = tok.tokenize("hundar");
        assert!(!tokens.is_empty(), "Swedish stemmer should produce tokens");
    }

    #[test]
    fn stemmer_norwegian_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Norwegian)
            .build()
            .unwrap();
        let tokens = tok.tokenize("hunder");
        assert!(
            !tokens.is_empty(),
            "Norwegian stemmer should produce tokens"
        );
    }

    #[test]
    fn stemmer_danish_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Danish)
            .build()
            .unwrap();
        let tokens = tok.tokenize("hunde");
        assert!(!tokens.is_empty(), "Danish stemmer should produce tokens");
    }

    #[test]
    fn stemmer_finnish_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Finnish)
            .build()
            .unwrap();
        let tokens = tok.tokenize("koirat");
        assert!(!tokens.is_empty(), "Finnish stemmer should produce tokens");
    }

    #[test]
    fn stemmer_hungarian_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Hungarian)
            .build()
            .unwrap();
        let tokens = tok.tokenize("kutyak");
        assert!(
            !tokens.is_empty(),
            "Hungarian stemmer should produce tokens"
        );
    }

    #[test]
    fn stemmer_romanian_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Romanian)
            .build()
            .unwrap();
        let tokens = tok.tokenize("pisici");
        assert!(!tokens.is_empty(), "Romanian stemmer should produce tokens");
    }

    #[test]
    fn stemmer_turkish_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Turkish)
            .build()
            .unwrap();
        let tokens = tok.tokenize("kediler");
        assert!(!tokens.is_empty(), "Turkish stemmer should produce tokens");
    }

    #[test]
    fn stemmer_arabic_known_stems() {
        let tok = Tokenizer::builder()
            .stemmer(Algorithm::Arabic)
            .build()
            .unwrap();
        let tokens = tok.tokenize("\u{0643}\u{062a}\u{0628}"); // "كتب"
        assert!(!tokens.is_empty(), "Arabic stemmer should produce tokens");
    }

    // ---------------------------------------------------------------
    // TEST-P1-006: Custom Function Override
    // ---------------------------------------------------------------

    #[test]
    fn custom_fn_completely_bypasses_builtin_pipeline() {
        // Even with stemmer and stopwords set, custom_fn should be the only thing used
        let tok = Tokenizer::builder()
            .language("english")
            .custom_fn(|_s| vec!["custom_token".to_string()])
            .build()
            .unwrap();
        let tokens = tok.tokenize("the running fox");
        assert_eq!(
            tokens,
            vec!["custom_token"],
            "custom_fn should override entire pipeline"
        );
    }

    #[test]
    fn custom_fn_receives_original_text() {
        let tok = Tokenizer::builder()
            .custom_fn(|s| vec![s.to_uppercase()])
            .build()
            .unwrap();
        let tokens = tok.tokenize("hello world");
        assert_eq!(tokens, vec!["HELLO WORLD"]);
    }

    // ---------------------------------------------------------------
    // TEST-P1-007: Empty Input Handling (additional cases)
    // ---------------------------------------------------------------

    #[test]
    fn whitespace_only_input_returns_empty() {
        let tok = Tokenizer::default();
        let tokens = tok.tokenize("   \t\n  ");
        assert!(
            tokens.is_empty(),
            "Whitespace-only input should produce empty token list"
        );
    }

    #[test]
    fn tokenize_with_vocab_whitespace_input() {
        let tok = Tokenizer::default();
        let mut vocab = HashMap::new();
        let result = tok.tokenize_with_vocab("   ", &mut vocab);
        assert!(result.0.is_empty());
        assert_eq!(result.1, 0);
        assert!(vocab.is_empty());
    }

    // ---------------------------------------------------------------
    // TEST-P1-005 continued: language() sets both stemmer and stopwords
    // ---------------------------------------------------------------

    #[test]
    fn language_spanish_sets_stemmer_and_stopwords() {
        let tok = Tokenizer::builder().language("spanish").build().unwrap();
        let tokens = tok.tokenize("el gato rapido");
        assert!(
            !tokens.contains(&"el".to_string()),
            "Spanish stopword 'el' should be removed"
        );
    }

    #[test]
    fn language_italian_sets_stemmer_and_stopwords() {
        let tok = Tokenizer::builder().language("italian").build().unwrap();
        let tokens = tok.tokenize("il gatto veloce");
        assert!(
            !tokens.contains(&"il".to_string()),
            "Italian stopword 'il' should be removed"
        );
    }

    #[test]
    fn language_unrecognized_returns_default_behavior() {
        let tok = Tokenizer::builder().language("klingon").build().unwrap();
        // Should still work, just without stopwords/stemmer
        let tokens = tok.tokenize("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn language_iso_code_works() {
        let tok = Tokenizer::builder().language("en").build().unwrap();
        let tokens = tok.tokenize("the running fox");
        assert!(
            !tokens.contains(&"the".to_string()),
            "ISO code 'en' should set English stopwords"
        );
    }

    #[test]
    fn tokenize_with_vocab_builds_vocabulary_correctly() {
        let tok = Tokenizer::default();
        let mut vocab = HashMap::new();
        let result = tok.tokenize_with_vocab("hello world hello", &mut vocab);
        assert_eq!(result.1, 3);
        assert_eq!(vocab.len(), 2, "Vocabulary should have 2 unique tokens");
        assert!(vocab.contains_key("hello"));
        assert!(vocab.contains_key("world"));
        // First occurrence of "hello" and "world" get IDs 0 and 1
        assert_eq!(result.0.len(), 3);
        // The two "hello" tokens should have the same ID
        assert_eq!(result.0[0], result.0[2]);
    }
}
