use std::convert::AsRef;
use std::fmt::Display;
use std::path::Path;

/// Defines a pattern that file paths are matched against.
#[derive(Clone, Debug)]
pub enum FilterPattern {
    /// A regex filter. Automatically rejects paths that aren't valid UTF-8.
    Regex(regex::Regex),
    /// A glob filter.
    Glob(glob::Pattern)
}

impl FilterPattern {
    // Tests whether or not the pattern matches the given path.
    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        match self {
            FilterPattern::Regex(regex) =>
            path.to_str().map_or(false, |str| regex.is_match(str)),
            FilterPattern::Glob(glob) => glob.matches_path(path),
        }
    }
}

impl Display for FilterPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterPattern::Regex(regex) => write!(f, "(Regex: {})", regex.as_str()),
            FilterPattern::Glob(glob) => write!(f, "(Glob: {})", glob.as_str()),
        }
    }
}

/// The action that is performed if a file matches a filter.
#[derive(Copy, Clone, Debug)]
pub enum FilterAction {
    /// The file is accepted.
    Accept,
    /// The file is rejected.
    Reject,
}

impl Display for FilterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterAction::Accept => write!(f, "Accept"),
            FilterAction::Reject => write!(f, "Reject"),
        }
    }
}

/// A filter that accepts or rejects a file if it matches a pattern.
#[derive(Clone, Debug)]
pub struct Filter {
    /// The pattern that file paths are matched against against.
    pattern: FilterPattern,
    /// The action that is taken if the file path matches this filter.
    /// The file is ignored otherwise.
    action: FilterAction,
}

impl Filter {
    /// Tests a file path against this filter, returning the specified
    /// if it maches, or None otherwise.
    pub fn test<P: AsRef<Path>>(&self, path: P) -> Option<FilterAction> {
        if self.pattern.is_match(path) {
            Some(self.action)
        } else {
            None
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.action, self.pattern)
    }
}

/// A ordered set of filters. A file path is checked against each filter in turn.
#[derive(Clone, Debug)]
pub struct FilterSet {
    /// The ordered list of filters to test candidates with.
    filters: Vec<Filter>,
    /// Indicates whether or not candidates are accepted even if they
    /// don't match any filters.
    pub accept_unmatched: bool,
}

impl FilterSet {
    /// Creates an empty filter set that rejects ignored matches by default.
    pub fn new() -> FilterSet {
        FilterSet { filters: Vec::new(), accept_unmatched: false }
    }

    /// Returns whether or not the file path passes all filters.
    pub fn is_acceptable<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        for filter in &self.filters {
            match filter.test(path) {
                Some(FilterAction::Accept) => return true,
                Some(FilterAction::Reject) => return false,
                None => continue,
            };
        }
        self.accept_unmatched
    }

    /// Creates and appends a filter that matches the provided regex with
    /// the given action.
    pub fn append_regex(&mut self, pattern: regex::Regex, action: FilterAction) {
        self.filters.push(Filter { pattern: FilterPattern::Regex(pattern), action })
    }

    /// Creates and appends a filter that matches the provided glob with
    /// the given action.
    pub fn append_glob(&mut self, pattern: glob::Pattern, action: FilterAction) {
        self.filters.push(Filter { pattern: FilterPattern::Glob(pattern), action })
    }
}

impl Default for FilterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for FilterSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} filters: \n", self.filters.len())?;
        for filter in &self.filters {
            writeln!(f, "\t{}", filter)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};
    use regex::Regex;

    use super::*;


    #[test]
    fn basic_regex_functionality() {
        let paths = vec![
            PathBuf::from("pizza/cheese/mozzarella"),
            PathBuf::from("pizza/cheese/parmesan"),
            PathBuf::from("sandwich/cheese/colby"),
            PathBuf::from("sandwich/cheese/provolone"),
        ];

        let any_pizza = regex::Regex::from_str("pizza").unwrap();
        let mut filter_set = FilterSet::new();
        filter_set.append_regex(any_pizza, FilterAction::Accept);

        assert!(filter_set.is_acceptable(&paths[0]));
        assert!(filter_set.is_acceptable(&paths[1]));
        assert!(!filter_set.is_acceptable(&paths[2]));
        assert!(!filter_set.is_acceptable(&paths[3]));
    }

    #[test]
    fn basic_glob_functionality() {
        let paths = vec![
            PathBuf::from("pizza/cheese/mozzarella"),
            PathBuf::from("pizza/cheese/parmesan"),
            PathBuf::from("sandwich/cheese/colby"),
            PathBuf::from("sandwich/cheese/provolone"),
        ];

        let any_pizza = glob::Pattern::from_str("pizza/*").unwrap();
        let mut filter_set = FilterSet::new();
        filter_set.append_glob(any_pizza, FilterAction::Accept);

        assert!(filter_set.is_acceptable(&paths[0]));
        assert!(filter_set.is_acceptable(&paths[1]));
        assert!(!filter_set.is_acceptable(&paths[2]));
        assert!(!filter_set.is_acceptable(&paths[3]));
    }

    #[test]
    fn empty_set_with_accepts_unmatched_true() {
        let path = PathBuf::from("pizza/cheese/mozzarella");
        let mut filter_set = FilterSet::new();
        filter_set.accept_unmatched = true;

        assert!(filter_set.is_acceptable(path));
    }

    #[test]
    fn empty_set_with_accepts_unmatched_false() {
        let path = PathBuf::from("pizza/cheese/mozzarella");
        let mut filter_set = FilterSet::new();
        filter_set.accept_unmatched = false;

        assert!(!filter_set.is_acceptable(path));
    }

    #[test]
    fn populated_set_with_accepts_unmatched_true() {
        let path = PathBuf::from("pizza/cheese/mozzarella");
        let mut filter_set = FilterSet::new();
        filter_set.accept_unmatched = true;

        filter_set.append_regex(Regex::from_str("garlic bread").unwrap(), FilterAction::Reject);
        filter_set.append_regex(Regex::from_str("spaghetti").unwrap(), FilterAction::Reject);

        assert!(filter_set.is_acceptable(path));
    }

    #[test]
    fn populated_set_with_accepts_unmatched_false() {
        let path = PathBuf::from("pizza/cheese/mozzarella");
        let mut filter_set = FilterSet::new();
        filter_set.accept_unmatched = false;

        filter_set.append_regex(Regex::from_str("garlic bread").unwrap(), FilterAction::Accept);
        filter_set.append_regex(Regex::from_str("spaghetti").unwrap(), FilterAction::Accept);

        assert!(!filter_set.is_acceptable(path));
    }
}
