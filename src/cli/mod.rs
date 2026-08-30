use clap::{Args, Parser, Subcommand};
use passkit::generator::passphrase::{self, PassphraseOptions};
use passkit::generator::{self, CharacterClasses, GenerateOptions};

#[derive(Parser)]
#[command(name = "passkit", version, about = "Password generator with a small local vault")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a random password
    Gen(GenArgs),
    /// Generate a diceware-style passphrase from the EFF wordlist
    Passphrase(PassphraseArgs),
}

#[derive(Args)]
struct GenArgs {
    /// Length of the generated password
    #[arg(long, default_value_t = 16)]
    length: usize,

    /// Number of passwords to generate
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Disable uppercase letters
    #[arg(long)]
    no_upper: bool,

    /// Disable lowercase letters
    #[arg(long)]
    no_lower: bool,

    /// Disable digits
    #[arg(long)]
    no_digits: bool,

    /// Disable symbols
    #[arg(long)]
    no_symbols: bool,

    /// Remove specific characters from the pool
    #[arg(long, value_name = "CHARS")]
    exclude: Option<String>,

    /// Drop look-alike characters (0/O, 1/l/I)
    #[arg(long)]
    no_ambiguous: bool,

    /// Guarantee at least one character from every enabled class
    #[arg(long)]
    require_each: bool,

    /// Generate from a template: U=upper, l=lower, d=digit, s=symbol,
    /// anything else is copied literally (e.g. "Ull-dddd-ss")
    #[arg(
        long,
        value_name = "PATTERN",
        conflicts_with_all = ["length", "require_each", "no_upper", "no_lower", "no_digits", "no_symbols"]
    )]
    pattern: Option<String>,
}

#[derive(Args)]
struct PassphraseArgs {
    /// Number of words
    #[arg(long, default_value_t = 6)]
    words: usize,

    /// Separator between words
    #[arg(long, default_value = "-")]
    separator: String,

    /// Capitalize the first letter of each word
    #[arg(long)]
    capitalize: bool,

    /// Number of passphrases to generate
    #[arg(long, default_value_t = 1)]
    count: usize,
}

impl PassphraseArgs {
    fn to_options(&self) -> PassphraseOptions {
        PassphraseOptions {
            words: self.words,
            separator: self.separator.clone(),
            capitalize: self.capitalize,
        }
    }
}

impl GenArgs {
    /// The full exclusion list: --exclude plus the ambiguous set.
    fn exclude_chars(&self) -> String {
        let mut exclude = self.exclude.clone().unwrap_or_default();
        if self.no_ambiguous {
            exclude.push_str(generator::AMBIGUOUS);
        }
        exclude
    }

    /// Translates CLI flags ("what to disable") into the library's
    /// domain model ("what is enabled").
    fn to_options(&self) -> GenerateOptions {
        GenerateOptions {
            length: self.length,
            classes: CharacterClasses {
                upper: !self.no_upper,
                lower: !self.no_lower,
                digits: !self.no_digits,
                symbols: !self.no_symbols,
            },
            exclude: self.exclude_chars(),
            require_each: self.require_each,
        }
    }
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Gen(args) => run_gen(&args),
        Command::Passphrase(args) => run_passphrase(&args),
    }
}

fn run_passphrase(args: &PassphraseArgs) {
    let options = args.to_options();

    for _ in 0..args.count {
        println!("{}", passphrase::generate(&options));
    }
}

fn run_gen(args: &GenArgs) {
    let options = args.to_options();

    for _ in 0..args.count {
        let result = match &args.pattern {
            Some(pattern) => generator::pattern::generate(pattern, &options.exclude),
            None => generator::generate(&options),
        };

        match result {
            Ok(password) => println!("{password}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
    }
}
