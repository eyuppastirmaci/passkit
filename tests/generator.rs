//! Integration tests: exercise the crate exactly as an external user would,
//! through the public API only.

use passkit::generator::passphrase::{self, PassphraseOptions};
use passkit::generator::{self, CharacterClasses, GenerateOptions};

#[test]
fn generates_a_password_with_defaults() {
    let password = generator::generate(&GenerateOptions::default()).unwrap();
    assert_eq!(password.len(), 16);
    assert!(password.is_ascii());
}

#[test]
fn no_ambiguous_workflow_drops_look_alikes() {
    // Mirrors what the CLI does for --no-ambiguous: append the exported
    // AMBIGUOUS set to the exclusion list.
    let options = GenerateOptions {
        length: 500,
        exclude: generator::AMBIGUOUS.to_string(),
        ..Default::default()
    };
    let password = generator::generate(&options).unwrap();
    assert!(password.chars().all(|c| !generator::AMBIGUOUS.contains(c)));
}

#[test]
fn digits_only_password() {
    let options = GenerateOptions {
        length: 6,
        classes: CharacterClasses {
            upper: false,
            lower: false,
            digits: true,
            symbols: false,
        },
        ..Default::default()
    };
    let pin = generator::generate(&options).unwrap();
    assert!(pin.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn pattern_generation_matches_the_template() {
    let password = generator::pattern::generate("Ull-dddd-ss", "").unwrap();
    assert_eq!(password.len(), 11);
    assert_eq!(&password[3..4], "-");
    assert_eq!(&password[8..9], "-");
    assert!(password[4..8].chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn passphrase_generation_with_options() {
    let options = PassphraseOptions {
        words: 4,
        separator: ".".to_string(),
        capitalize: true,
    };
    let phrase = passphrase::generate(&options);
    let words: Vec<&str> = phrase.split('.').collect();
    assert_eq!(words.len(), 4);
    assert!(words.iter().all(|w| w.chars().next().unwrap().is_ascii_uppercase()));
}

#[test]
fn errors_are_printable() {
    let options = GenerateOptions {
        classes: CharacterClasses {
            upper: false,
            lower: false,
            digits: false,
            symbols: false,
        },
        ..Default::default()
    };
    let error = generator::generate(&options).unwrap_err();
    assert!(!error.to_string().is_empty());
}
