use std::io::stdout;
use std::path::Path;

use clap::{Parser, Subcommand};

use analyzer::Analyzer;

/// Command line arguments configuration.
#[derive(Parser, Debug)]
#[clap(name = "Analyze")]
#[clap(author = "Aiko Wessels <aiko.wessels@gmail.com>")]
#[clap(version = "0.1.0")]
#[clap(about = "Analyze regex patterns inside a file.")]
struct Cli {
    /// Subcommand
    #[clap(subcommand)]
    command: Commands,
}

/// Subcommands are stored in this enum.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Find actual matches and count the matches for each regex.
    Match {
        /// Path to the patterns file.
        #[clap(short, long)]
        patterns: String,

        /// Path to the input file.
        files: Vec<String>,

        /// Comma separated list of patterns to include.
        #[clap(long)]
        include: Option<String>,

        /// Comma separated list of patterns to exclude.
        #[clap(long)]
        exclude: Option<String>,

        /// If displaying matches, show only the top n matches.
        #[clap(short, long, default_value_t = usize::MAX)]
        top: usize,
    },

    /// Count the number of matches for each regex, but do not save the actual returned matches.
    Count {
        /// YAML file containing the list of patterns to search.
        #[clap(short, long)]
        patterns: String,

        /// Path to the input file.
        files: Vec<String>,

        /// Comma separated list of patterns to include.
        #[clap(long)]
        include: Option<String>,

        /// Comma separated list of patterns to exclude.
        #[clap(long)]
        exclude: Option<String>,
    },

    /// Clean the files by removing and replacing.
    Clean {
        /// Path to the YAML configuration file.
        #[clap(short, long)]
        patterns: String,

        /// Path to the input file.
        files: Vec<String>,
    },

    /// Analyze the entire vocab of the source file.
    Vocab {
        /// Path to the input file.
        files: Vec<String>,
    },
}

/// Main entry point.
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Match {
            patterns,
            files,
            include: _,
            exclude: _,
            top,
        } => {
            let patterns = analyzer::parse_input(&patterns).unwrap();

            for file in files {
                let file = Path::new(&file);
                let mut matcher = analyzer::match_file(file, patterns.clone(), *top).unwrap();
                println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
                matcher.format(&mut stdout());
                println!();
            }
        }
        Commands::Count {
            patterns,
            files,
            include: _,
            exclude: _,
        } => {
            let patterns = analyzer::parse_input(&patterns).unwrap();
            for file in files {
                let file = Path::new(&file);
                let mut counter = analyzer::count_file(file, patterns.clone()).unwrap();
                println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
                counter.format(&mut stdout());
                println!();
            }
        }
        Commands::Clean {
            patterns: _,
            files: _,
        } => {}
        Commands::Vocab { files } => {
            for file in files {
                let file = Path::new(&file);
                let mut vocabulizer = analyzer::vocabulizer(file).unwrap();
                println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
                vocabulizer.format(&mut stdout());
                println!();
            }
        }
    }
}
