use clap::{Args, Parser, Subcommand};
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
}

impl GenArgs {
    /// Translates CLI flags ("what to disable") into the library's
    /// domain model ("what is enabled").
    fn to_options(&self) -> GenerateOptions {
        let mut exclude = self.exclude.clone().unwrap_or_default();
        if self.no_ambiguous {
            exclude.push_str(generator::AMBIGUOUS);
        }

        GenerateOptions {
            length: self.length,
            classes: CharacterClasses {
                upper: !self.no_upper,
                lower: !self.no_lower,
                digits: !self.no_digits,
                symbols: !self.no_symbols,
            },
            exclude,
            require_each: self.require_each,
        }
    }
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Gen(args) => run_gen(&args),
    }
}

fn run_gen(args: &GenArgs) {
    let options = args.to_options();

    for _ in 0..args.count {
        match generator::generate(&options) {
            Ok(password) => println!("{password}"),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
    }
}
