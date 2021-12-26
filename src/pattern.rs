use std::{collections::HashMap, io::Write};

use num_format::{Locale, ToFormattedString};
use regex::Regex;

use self::group::GroupVec;

/// Actual pattern instance, which holds its name and its regex.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub regex: Regex,
}

pub mod group {
    //! Contains method related to the GroupTree data structure.
    //!
    use std::ops::{Deref, DerefMut};

    /// Represents a generic tree which contains named groups.
    #[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
    pub enum GroupTree<T> {
        Leaf(T),
        Group {
            name: String,
            group: Vec<GroupTree<T>>,
        },
    }

    #[derive(Debug, Clone)]
    pub struct GroupVec<V> {
        /// Holds references to the flattened vec, as well as to the original tree, so that we can
        /// recreate the groups later.
        pub inner: Vec<GroupTree<usize>>,

        /// A flattened list of references to the tree.
        pub flattened: Vec<V>,
    }

    impl<V> GroupVec<V> {
        pub fn from_tree<T>(tree_vec: Vec<GroupTree<T>>) -> GroupVec<V>
        where
            V: From<T>,
        {
            /// Helper function that traverses the GroupTree and consumes it, creating the
            /// GroupVec.
            fn traverse<T, V>(tree: GroupTree<T>, vec: &mut Vec<V>) -> GroupTree<usize>
            where
                V: From<T>,
            {
                match tree {
                    GroupTree::Leaf(other) => {
                        let value = V::from(other);
                        vec.push(value);

                        GroupTree::Leaf(vec.len() - 1)
                    }
                    GroupTree::Group { name, group } => {
                        let mut inner_group: Vec<GroupTree<usize>> = Vec::new();
                        for item in group {
                            inner_group.push(traverse(item, vec));
                        }

                        inner_group.sort();

                        GroupTree::Group {
                            name,
                            group: inner_group,
                        }
                    }
                }
            }

            let mut flattened = Vec::new();
            let mut inner = Vec::new();
            for tree in tree_vec {
                inner.push(traverse(tree, &mut flattened));
            }
            inner.sort();

            GroupVec { inner, flattened }
        }
    }

    impl<V> Deref for GroupVec<V> {
        type Target = [V];

        fn deref(&self) -> &Self::Target {
            &self.flattened
        }
    }

    impl<V> DerefMut for GroupVec<V> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.flattened
        }
    }

    /// Trait used to convert between two types when creating the GroupVec from the GroupTree structs.
    pub trait From<T> {
        fn from(other: T) -> Self;
    }
}

pub trait Analyzer<T> {
    type Analysis;

    /// Perform analysis on the given line.
    fn analyze(&mut self, line: String);

    /// Writes the results to the given writer.
    fn format<W>(&mut self, writer: &mut W)
    where
        W: Write;
}

pub mod counter {
    use super::group::*;
    use super::Analyzer;
    use super::*;

    #[derive(Debug, Clone)]
    struct Inner {
        pattern: Pattern,
        count: u64,
    }

    impl group::From<Pattern> for Inner {
        fn from(other: Pattern) -> Self {
            Inner {
                pattern: other,
                count: 0_u64,
            }
        }
    }

    #[derive(Debug)]
    pub struct PatternCounter {
        patterns: GroupVec<Inner>,
    }

    impl PatternCounter {
        pub fn new(tree: Vec<GroupTree<Pattern>>) -> Self {
            PatternCounter {
                patterns: GroupVec::from_tree::<Pattern>(tree),
            }
        }
    }

    impl Analyzer<Pattern> for PatternCounter {
        type Analysis = u64;

        fn analyze(&mut self, line: String) {
            for inner in &mut self.patterns[..] {
                if inner.pattern.regex.is_match(&line) {
                    inner.count += 1;
                }
            }
        }

        fn format<W>(&mut self, _writer: &mut W)
        where
            W: Write,
        {
            // Find longest name and count
            let mut longest_name = 0;
            let mut longest_count = 0;

            for inner in &self.patterns[..] {
                let name_len = inner.pattern.name.chars().count();
                if longest_name < name_len {
                    longest_name = name_len;
                }

                let count_len = inner.count.to_formatted_string(&Locale::en).chars().count();
                if longest_count < count_len {
                    longest_count = count_len;
                }
            }

            fn traverse(tree: &GroupTree<usize>, slice: &[Inner], indent: usize) {
                match tree {
                    GroupTree::Leaf(index) => {
                        let Inner { pattern, count } = slice.get(*index).unwrap();
                        println!(
                            "{: <indent$}{:} {:}",
                            "",
                            format!("{}:", pattern.name),
                            count.to_formatted_string(&Locale::en),
                            indent = indent
                        );
                    }
                    GroupTree::Group { name, group } => {
                        println!("{}:", name);
                        for inner_tree in group {
                            traverse(inner_tree, slice, indent + 2);
                        }
                    }
                }
            }

            for group_tree in &self.patterns.inner {
                traverse(group_tree, &self.patterns[..], 0)
            }
        }
    }
}

pub mod matcher {
    use super::Analyzer;
    use super::*;
    use crate::GroupTree;

    #[derive(Debug, Clone)]
    struct Inner {
        pub pattern: Pattern,
        pub matches: HashMap<String, u64>,
    }

    impl group::From<Pattern> for Inner {
        fn from(other: Pattern) -> Self {
            Inner {
                pattern: other,
                matches: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct PatternMatcher {
        patterns: GroupVec<Inner>,
        top: usize,
    }

    impl PatternMatcher {
        pub fn new(tree: Vec<GroupTree<Pattern>>, top: usize) -> Self {
            PatternMatcher {
                patterns: GroupVec::from_tree::<Pattern>(tree),
                top,
            }
        }
    }

    impl Analyzer<Pattern> for PatternMatcher {
        type Analysis = HashMap<String, u64>;

        fn analyze(&mut self, line: String) {
            for inner in &mut self.patterns[..] {
                for mat in inner.pattern.regex.find_iter(&line) {
                    let entry = inner.matches.entry(mat.as_str().to_string()).or_insert(0);
                    *entry += 1;
                }
            }
        }

        fn format<W>(&mut self, _writer: &mut W)
        where
            W: std::io::Write,
        {
            // Find longest name and count
            let mut longest_name = 0;
            let mut longest_match = 0;
            let mut longest_count = 0;
            for inner in &self.patterns[..] {
                let name_len = inner.pattern.name.chars().count();
                if longest_name < name_len {
                    longest_name = name_len;
                }

                for (_mat, count) in inner.matches.iter().take(self.top) {
                    let count_len = count.to_formatted_string(&Locale::en).chars().count();
                    if longest_count < count_len {
                        longest_count = count_len;
                    }
                    let match_len = inner.pattern.name.chars().count();
                    if longest_match < match_len {
                        longest_match = match_len;
                    }
                }
            }

            for inner in &self.patterns[..] {
                println!("{}", inner.pattern.name);
                // TODO: Sort data before printing.
                for (mat, count) in inner.matches.iter().take(self.top) {
                    println!(
                        "\t{:<match_len$} {} {:>count_len$}",
                        format!("{}:", mat),
                        count.to_formatted_string(&Locale::en),
                        match_len = longest_match + 1,
                        count_len = longest_count
                    );
                }
            }
        }
    }
}
