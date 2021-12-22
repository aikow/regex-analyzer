use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;

use clap::Parser;
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use serde::{Serialize, Deserialize};


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
    #[clap(short = 'c', long)]
    patterns_file: String,

    /// Optional comma separated list of patterns to use from the patterns file.
    #[clap(short, long)]
    patterns: Option<String>,
}


/// Represents a single pattern to search.
#[derive(Clone)]
struct Pattern {
    name: String,
    pattern: Regex,
    count: usize,
}


impl Pattern {
    /// Create a new pattern if the passed string is a valid regex, otherwise, return an error.
    fn new(name: String, pattern_string: String) -> Result<Pattern, String> {
        let pattern = Regex::new(&pattern_string)
            .map_err(|e| format!("Unable to create regex from {}: {}", pattern_string, e))?;

        Ok(Pattern { name, pattern, count: 0, })
    }
}


impl Pattern {
    /// Read a list of patterns from a YAML file.
    fn parse_from_file<P>(path: P) -> Result<Vec<Pattern>, String>
        where P: AsRef<Path>
    {
        /// Represents the form of a pattern as it is read from the configuration file.
        #[derive(Serialize, Deserialize)]
        struct PatternHelper {
            name: String,
            pattern: String,
        }

        let file = File::open(path).map_err(|e| format!("{}", e))?;
        let reader = BufReader::new(file);

        let string_patterns: HashMap<String, String> = serde_yaml::from_reader(reader)
            .map_err(|e| format!("{}", e))?;

         string_patterns
            .into_iter()
            .map(|(name, pattern)| {
                Pattern::new(name, pattern)
            }).collect()
    }
}


fn analyze_file<P>(path: P, mut patterns: Vec<Pattern>) -> Result<Vec<Pattern>, String>
        where P: AsRef<Path> {
    let file = File::open(path).map_err(|e| format!("{}", e))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        for Pattern { pattern, count, .. } in &mut patterns {
            if pattern.is_match(&line) {
                *count += 1;
            }
        }
    }

    Ok(patterns)
}


fn print_results(patterns: &mut Vec<Pattern>) {
    let mut longest_name = 0;
    let mut longest_count = 0;
    for Pattern { name, count, .. } in patterns.iter() {
        let name_len = name.chars().count();
        if longest_name < name_len {
            longest_name = name_len;
        }

        let count_len = count.to_formatted_string(&Locale::en).chars().count();
        if longest_count < count_len {
            longest_count = count_len;
        }
    }

    patterns.sort_by(|lhs, rhs| rhs.count.cmp(&lhs.count));

    for Pattern { name, count, .. } in patterns {
        println!(
            "{:<name_len$} {:>count_len$}",
            format!("{}:", name),
            count.to_formatted_string(&Locale::en),
            name_len=longest_name + 1,
            count_len=longest_count
        );
    }
}

fn main() {
    let cli = Cli::parse();

    // Read patterns from the given file.
    let patterns = Pattern::parse_from_file(&cli.patterns_file).unwrap();

    let patterns: Vec<_> = if let Some(filter) = cli.patterns {
        let filter: HashSet<&str> = filter.split(',').collect();
        patterns
            .into_iter()
            .filter(|Pattern { name, .. }| filter.contains(&name[..]))
            .collect()
    } else {
        patterns
    };

    for file in cli.files {
        let file = Path::new(&file);
        let mut patterns = analyze_file(file, patterns.to_vec()).unwrap() ;

        // println!("{:=^length$}", format!(" {} ", file.file_name().unwrap().to_str().unwrap()), length=20);
        println!("==== {} ====", file.file_name().unwrap().to_str().unwrap());
        print_results(&mut patterns);
        println!();
    }
}
