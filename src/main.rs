use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, stdout};
use std::path::Path;

use clap::Parser;

/// Command line arguments configuration.
#[derive(Parser, Debug)]
#[clap(name = "Analyze")]
#[clap(author = "Aiko Wessels <aiko.wessels@gmail.com>")]
#[clap(version = "0.1.0")]
#[clap(about = "Analyze regex patterns inside a file.")]
struct Cli {
    /// Path to the input file.
    files: Vec<String>,

    /// Path to the patterns file.
    #[clap(short = 's', long)]
    patterns_file: String,

    /// Optional comma separated list of patterns to use from the patterns file.
    #[clap(short, long)]
    patterns: Option<String>,

    /// List the found matches by their frequencies.
    #[clap(short, long)]
    matches: bool,

    /// If displaying matches, show only the top n matches.
    #[clap(short, long, default_value_t = usize::MAX)]
    top_matches: usize,
}


/// Main entry point.
fn main() {
    let cli = Cli::parse();

    // Read patterns from the given file.
    let patterns = analyzer::parse_input(&cli.patterns_file).unwrap();

    // let patterns: Vec<_> = if let Some(filter) = cli.patterns {
    //     let filter: HashSet<&str> = filter.split(',').collect();
    //     patterns
    //         .into_iter()
    //         .filter(|Pattern { name, .. }| filter.contains(&name[..]))
    //         .collect()
    // } else {
    //     patterns
    // };

    for file in cli.files {
        let file = Path::new(&file);
        if cli.matches {
            let mut matcher = analyzer::match_file(file, patterns.clone(), cli.top_matches).unwrap();
            println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
            matcher.format(&mut stdout());
            println!();
        } else {
            let mut counter = analyzer::count_file(file, patterns.clone()).unwrap();
            println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
            counter.format(&mut stdout());
            println!();
        }
    }
}
