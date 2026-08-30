mod charset;
pub mod passphrase;
pub mod pattern;

pub use charset::{AMBIGUOUS, CharacterClasses};

use rand::RngExt;
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub length: usize,
    pub classes: CharacterClasses,
    /// Characters removed from the pool even if their class is enabled.
    pub exclude: String,
    /// Guarantee at least one character from every enabled class.
    pub require_each: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            length: 16,
            classes: CharacterClasses::default(),
            exclude: String::new(),
            require_each: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GeneratorError {
    /// Every class is disabled, or every character was excluded.
    EmptyCharset,
    /// A class is enabled but all of its characters are excluded,
    /// so `require_each` can never be satisfied.
    EmptyClass { class: &'static str },
    /// `require_each` needs at least one slot per enabled class.
    LengthTooShort { length: usize, required: usize },
}

impl std::fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCharset => {
                write!(f, "character pool is empty: enable at least one class or exclude fewer characters")
            }
            Self::EmptyClass { class } => {
                write!(f, "class '{class}' is enabled but all of its characters are excluded")
            }
            Self::LengthTooShort { length, required } => {
                write!(f, "length {length} is too short for require-each: need at least {required}")
            }
        }
    }
}

impl std::error::Error for GeneratorError {}

pub fn generate(options: &GenerateOptions) -> Result<String, GeneratorError> {
    let pools = charset::class_pools(&options.classes, &options.exclude);

    if options.require_each {
        if let Some((name, _)) = pools.iter().find(|(_, pool)| pool.is_empty()) {
            return Err(GeneratorError::EmptyClass { class: name });
        }
        if options.length < pools.len() {
            return Err(GeneratorError::LengthTooShort {
                length: options.length,
                required: pools.len(),
            });
        }
    }

    let combined: Vec<char> = pools.iter().flat_map(|(_, pool)| pool.iter().copied()).collect();
    if combined.is_empty() {
        return Err(GeneratorError::EmptyCharset);
    }

    let mut rng = UnwrapErr(SysRng);

    // Rejection sampling: draw a full candidate uniformly, retry until every
    // enabled class is present. Unlike "place one of each, then shuffle",
    // this keeps the character distribution unbiased.
    loop {
        let candidate: String = (0..options.length)
            .map(|_| {
                // random_range avoids modulo bias via rejection sampling;
                // a naive `random % len` would favor some characters and lower entropy.
                combined[rng.random_range(0..combined.len())]
            })
            .collect();

        if !options.require_each
            || pools.iter().all(|(_, pool)| candidate.chars().any(|c| pool.contains(&c)))
        {
            return Ok(candidate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_requested_length() {
        let options = GenerateOptions { length: 20, ..Default::default() };
        assert_eq!(generate(&options).unwrap().len(), 20);
    }

    #[test]
    fn class_toggles_restrict_output() {
        let options = GenerateOptions {
            length: 200,
            classes: CharacterClasses {
                upper: true,
                lower: true,
                digits: false,
                symbols: false,
            },
            ..Default::default()
        };
        let password = generate(&options).unwrap();
        assert!(password.chars().all(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn exclude_removes_characters() {
        let options = GenerateOptions {
            length: 500,
            exclude: "aA0!".to_string(),
            ..Default::default()
        };
        let password = generate(&options).unwrap();
        assert!(!password.contains(['a', 'A', '0', '!']));
    }

    #[test]
    fn require_each_covers_every_enabled_class() {
        let options = GenerateOptions {
            length: 4,
            require_each: true,
            ..Default::default()
        };
        // Length 4 with 4 classes forces exactly one character per class,
        // so a single missing class would make this fail.
        for _ in 0..20 {
            let password = generate(&options).unwrap();
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_lowercase()));
            assert!(password.chars().any(|c| c.is_ascii_digit()));
            assert!(password.chars().any(|c| !c.is_ascii_alphanumeric()));
        }
    }

    #[test]
    fn default_length_is_16() {
        let password = generate(&GenerateOptions::default()).unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn consecutive_passwords_differ() {
        // With a ~90-char pool and length 16 a collision is astronomically
        // unlikely; equality here would mean the RNG is broken.
        let options = GenerateOptions::default();
        assert_ne!(generate(&options).unwrap(), generate(&options).unwrap());
    }

    #[test]
    fn require_each_works_with_a_subset_of_classes() {
        let options = GenerateOptions {
            length: 2,
            classes: CharacterClasses {
                upper: true,
                lower: false,
                digits: true,
                symbols: false,
            },
            require_each: true,
            ..Default::default()
        };
        for _ in 0..20 {
            let password = generate(&options).unwrap();
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn fully_excluded_pool_is_empty_charset() {
        let options = GenerateOptions {
            classes: CharacterClasses {
                upper: false,
                lower: false,
                digits: true,
                symbols: false,
            },
            exclude: "0123456789".to_string(),
            ..Default::default()
        };
        assert_eq!(generate(&options), Err(GeneratorError::EmptyCharset));
    }

    #[test]
    fn all_classes_disabled_is_an_error() {
        let options = GenerateOptions {
            classes: CharacterClasses {
                upper: false,
                lower: false,
                digits: false,
                symbols: false,
            },
            ..Default::default()
        };
        assert_eq!(generate(&options), Err(GeneratorError::EmptyCharset));
    }

    #[test]
    fn require_each_rejects_too_short_length() {
        let options = GenerateOptions {
            length: 3,
            require_each: true,
            ..Default::default()
        };
        assert_eq!(
            generate(&options),
            Err(GeneratorError::LengthTooShort { length: 3, required: 4 })
        );
    }

    #[test]
    fn require_each_rejects_fully_excluded_class() {
        let options = GenerateOptions {
            length: 10,
            exclude: "0123456789".to_string(),
            require_each: true,
            ..Default::default()
        };
        assert_eq!(
            generate(&options),
            Err(GeneratorError::EmptyClass { class: "digits" })
        );
    }
}
