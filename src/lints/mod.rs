mod double_entry;
mod duplicates;
mod unbalanced;

use std::fmt::Display;

pub use double_entry::find_double_entries;
pub use duplicates::find_duplicates;
pub use unbalanced::find_unbalanced_entries;

#[derive(Debug)]
pub enum Lint<'a> {
    DoubleEntry(double_entry::DoubleEntry<'a>),
    DuplicateTransaction(duplicates::DuplicateTransaction<'a>),
    UnbalancedEntry(unbalanced::UnbalancedEntry<'a>),
}

impl<'a> Display for Lint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lint::DoubleEntry(inner) => write!(f, "{}", inner),
            Lint::DuplicateTransaction(inner) => write!(f, "{}", inner),
            Lint::UnbalancedEntry(inner) => write!(f, "{}", inner),
        }
    }
}
