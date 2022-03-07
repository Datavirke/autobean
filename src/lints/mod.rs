mod double_entry;
mod duplicates;

use std::fmt::Display;

pub use double_entry::find_double_entries;
pub use duplicates::find_duplicates;

pub enum Lint<'a> {
    DoubleEntry(double_entry::DoubleEntry<'a>),
    DuplicateTransaction(duplicates::DuplicateTransaction<'a>),
}

impl<'a> Display for Lint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lint::DoubleEntry(inner) => write!(f, "{}", inner),
            Lint::DuplicateTransaction(inner) => write!(f, "{}", inner),
        }
    }
}
