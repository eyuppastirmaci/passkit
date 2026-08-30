use std::sync::LazyLock;

use rand::RngExt;
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;

/// EFF long wordlist (7776 words), embedded into the binary at compile
/// time. Each line is "<dice roll>\t<word>"; only the word is kept.
/// Parsed once on first access.
static WORDLIST: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    include_str!("../../assets/eff_large_wordlist.txt")
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .collect()
});

#[derive(Debug, Clone)]
pub struct PassphraseOptions {
    pub words: usize,
    pub separator: String,
    /// Capitalize the first letter of each word.
    pub capitalize: bool,
}

impl Default for PassphraseOptions {
    fn default() -> Self {
        Self {
            words: 6,
            separator: "-".to_string(),
            capitalize: false,
        }
    }
}

pub fn generate(options: &PassphraseOptions) -> String {
    let mut rng = UnwrapErr(SysRng);

    let words: Vec<String> = (0..options.words)
        .map(|_| {
            let word = WORDLIST[rng.random_range(0..WORDLIST.len())];
            if options.capitalize {
                capitalize(word)
            } else {
                word.to_string()
            }
        })
        .collect();

    words.join(&options.separator)
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        // char::to_uppercase returns an iterator: uppercasing one char can yield several (e.g. 'ß' → "SS").
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordlist_has_all_eff_words() {
        assert_eq!(WORDLIST.len(), 7776);
    }

    #[test]
    fn wordlist_contains_words_not_dice_rolls() {
        assert!(WORDLIST.iter().all(|w| w.chars().all(|c| !c.is_ascii_digit())));
    }

    #[test]
    fn generates_requested_word_count() {
        let options = PassphraseOptions::default();
        let passphrase = generate(&options);
        assert_eq!(passphrase.split('-').count(), 6);
    }

    #[test]
    fn zero_words_gives_empty_string() {
        let options = PassphraseOptions { words: 0, ..Default::default() };
        assert_eq!(generate(&options), "");
    }

    #[test]
    fn words_come_from_the_wordlist() {
        let options = PassphraseOptions { words: 10, ..Default::default() };
        let passphrase = generate(&options);
        for word in passphrase.split('-') {
            assert!(WORDLIST.contains(&word), "unexpected word: {word}");
        }
    }

    #[test]
    fn custom_separator_is_used() {
        let options = PassphraseOptions {
            words: 4,
            separator: " ".to_string(),
            ..Default::default()
        };
        assert_eq!(generate(&options).split(' ').count(), 4);
    }

    #[test]
    fn capitalize_uppercases_each_word() {
        let options = PassphraseOptions {
            words: 8,
            capitalize: true,
            ..Default::default()
        };
        let passphrase = generate(&options);
        for word in passphrase.split('-') {
            assert!(word.chars().next().unwrap().is_ascii_uppercase());
        }
    }

    #[test]
    fn consecutive_passphrases_differ() {
        // 6 words from 7776 gives ~77.5 bits; a collision means a broken RNG.
        let options = PassphraseOptions::default();
        assert_ne!(generate(&options), generate(&options));
    }
}
