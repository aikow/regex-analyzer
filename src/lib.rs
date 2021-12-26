use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

pub mod pattern;

pub use crate::pattern::counter::PatternCounter;
pub use crate::pattern::group::GroupTree;
pub use crate::pattern::matcher::PatternMatcher;
pub use crate::pattern::{Analyzer, Pattern};

pub fn count_file<P>(path: P, tree: Vec<GroupTree<Pattern>>) -> Result<PatternCounter, String>
where
    P: AsRef<Path>,
{
    let mut counter = PatternCounter::new(tree);

    let file = File::open(path).map_err(|e| format!("{}", e))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        counter.analyze(line);
    }

    Ok(counter)
}

pub fn match_file<P>(
    path: P,
    tree: Vec<GroupTree<Pattern>>,
    top: usize,
) -> Result<PatternMatcher, String>
where
    P: AsRef<Path>,
{
    let mut matches = PatternMatcher::new(tree, top);

    let file = File::open(path).map_err(|e| format!("{}", e))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        matches.analyze(line);
    }

    Ok(matches)
}

pub fn parse_input<P>(path: P) -> Result<Vec<GroupTree<Pattern>>, String>
where
    P: AsRef<Path>,
{
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum PatternTreeHelper {
        /// Contains just a regex.
        Leaf(String),

        /// Contains a map from a name to either a regex or to another sub-group.
        Node(HashMap<String, PatternTreeHelper>),
    }

    let file = File::open(path).map_err(|e| format!("{}", e))?;
    let reader = BufReader::new(file);
    let pattern_tree: HashMap<String, PatternTreeHelper> =
        serde_yaml::from_reader(reader).expect("Failed to parse YAML");

    fn traverse(name: String, tree: &PatternTreeHelper) -> Result<GroupTree<Pattern>, String> {
        match tree {
            PatternTreeHelper::Leaf(pattern) => {
                let regex = Regex::new(pattern).map_err(|e| format!("{}", e))?;
                Ok(GroupTree::Leaf(Pattern { name, regex }))
            }
            PatternTreeHelper::Node(map) => {
                let (patterns, invalid): (Vec<_>, Vec<_>) = map
                    .iter()
                    .map(|(name, helper)| traverse(name.clone(), helper))
                    .partition(Result::is_ok);
                let patterns: Vec<_> = patterns.into_iter().map(Result::unwrap).collect();
                let invalid: Vec<_> = invalid.into_iter().map(Result::unwrap).collect();
                if !invalid.is_empty() {
                    return Err(format!(
                        "Unable to convert the following patterns: {:?}",
                        invalid
                    ));
                }

                Ok(GroupTree::Group {
                    name,
                    group: patterns,
                })
            }
        }
    }

    pattern_tree
        .into_iter()
        .map(|(name, helper)| traverse(name, &helper))
        .collect()
}
