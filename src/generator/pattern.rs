use rand::RngExt;
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;

use super::GeneratorError;
use super::charset::ClassKind;

/// One position in a pattern template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternToken {
    /// A placeholder: draw a random character from this class.
    Class(ClassKind),
    /// Any other character: copied to the output as-is.
    Literal(char),
}

fn parse(pattern: &str) -> Vec<PatternToken> {
    pattern
        .chars()
        .map(|c| match c {
            'U' => PatternToken::Class(ClassKind::Upper),
            'l' => PatternToken::Class(ClassKind::Lower),
            'd' => PatternToken::Class(ClassKind::Digits),
            's' => PatternToken::Class(ClassKind::Symbols),
            other => PatternToken::Literal(other),
        })
        .collect()
}

/// Generates a password from a template where `U` = uppercase, `l` =
/// lowercase, `d` = digit, `s` = symbol and every other character is
/// copied literally. `exclude` narrows the class pools, same as in
/// [`super::generate`].
pub fn generate(pattern: &str, exclude: &str) -> Result<String, GeneratorError> {
    let pools: Vec<Vec<char>> = ClassKind::ALL.iter().map(|kind| kind.pool(exclude)).collect();
    let mut rng = UnwrapErr(SysRng);

    parse(pattern)
        .into_iter()
        .map(|token| match token {
            PatternToken::Literal(c) => Ok(c),
            PatternToken::Class(kind) => {
                // Safe to index by discriminant: ALL is declared in discriminant order (tested in charset.rs).
                let pool = &pools[kind as usize];
                if pool.is_empty() {
                    return Err(GeneratorError::EmptyClass { class: kind.name() });
                }
                // random_range avoids modulo bias via rejection sampling.
                Ok(pool[rng.random_range(0..pool.len())])
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_placeholders_and_literals() {
        assert_eq!(
            parse("Ul-d s"),
            [
                PatternToken::Class(ClassKind::Upper),
                PatternToken::Class(ClassKind::Lower),
                PatternToken::Literal('-'),
                PatternToken::Class(ClassKind::Digits),
                PatternToken::Literal(' '),
                PatternToken::Class(ClassKind::Symbols),
            ]
        );
    }

    #[test]
    fn literals_are_copied_verbatim() {
        assert_eq!(generate("-- __ ??", "").unwrap(), "-- __ ??");
    }

    #[test]
    fn placeholders_draw_from_their_class() {
        let password = generate("Ulds", "").unwrap();
        let chars: Vec<char> = password.chars().collect();
        assert!(chars[0].is_ascii_uppercase());
        assert!(chars[1].is_ascii_lowercase());
        assert!(chars[2].is_ascii_digit());
        assert!(!chars[3].is_ascii_alphanumeric());
    }

    #[test]
    fn exclude_narrows_the_pool() {
        // Excluding every digit but '7' forces the output.
        let password = generate("dddd", "01234568 9").unwrap();
        assert_eq!(password, "7777");
    }

    #[test]
    fn fully_excluded_class_is_an_error() {
        assert_eq!(
            generate("Ud", "0123456789"),
            Err(GeneratorError::EmptyClass { class: "digits" })
        );
    }

    #[test]
    fn empty_pattern_gives_empty_string() {
        assert_eq!(generate("", "").unwrap(), "");
    }
}
