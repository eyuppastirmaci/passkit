const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.<>?";

/// Look-alike characters dropped by `--no-ambiguous`.
pub const AMBIGUOUS: &str = "0O1lI";

/// Which character classes are enabled for generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterClasses {
    pub upper: bool,
    pub lower: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for CharacterClasses {
    fn default() -> Self {
        Self {
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
        }
    }
}

/// Returns one `(name, pool)` pair per *enabled* class, with excluded
/// characters already removed. Pools are kept separate (instead of one
/// merged charset) so `require_each` can check membership per class.
pub(crate) fn class_pools(
    classes: &CharacterClasses,
    exclude: &str,
) -> Vec<(&'static str, Vec<char>)> {
    let all = [
        ("upper", classes.upper, UPPER),
        ("lower", classes.lower, LOWER),
        ("digits", classes.digits, DIGITS),
        ("symbols", classes.symbols, SYMBOLS),
    ];

    all.into_iter()
        .filter(|(_, enabled, _)| *enabled)
        .map(|(name, _, set)| {
            let pool = set.chars().filter(|c| !exclude.contains(*c)).collect();
            (name, pool)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_classes_are_omitted() {
        let classes = CharacterClasses {
            upper: true,
            lower: false,
            digits: true,
            symbols: false,
        };
        let pools = class_pools(&classes, "");
        let names: Vec<&str> = pools.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, ["upper", "digits"]);
    }

    #[test]
    fn excluded_characters_are_removed_from_their_pool() {
        let pools = class_pools(&CharacterClasses::default(), "ABC012");
        let upper = &pools[0].1;
        let digits = &pools[2].1;
        assert_eq!(upper.len(), 23);
        assert_eq!(digits.len(), 7);
        assert!(!upper.contains(&'A'));
        assert!(!digits.contains(&'0'));
    }

    #[test]
    fn ambiguous_set_only_contains_look_alikes() {
        let pools = class_pools(&CharacterClasses::default(), AMBIGUOUS);
        for (_, pool) in pools {
            assert!(pool.iter().all(|c| !AMBIGUOUS.contains(*c)));
        }
    }
}
