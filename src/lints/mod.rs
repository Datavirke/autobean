mod appendix_missing;
mod double_entry;
mod duplicate_appendix;
mod duplicates;
mod sequential_appendix;
mod unbalanced;

use std::fmt::Display;

pub use appendix_missing::find_missing_appendices;
pub use double_entry::find_double_entries;
pub use duplicate_appendix::find_duplicate_appendix_ids;
pub use duplicates::find_duplicates;
pub use sequential_appendix::find_nonsequential_appendices;
pub use unbalanced::find_unbalanced_entries;

#[derive(Debug)]
pub enum Lint<'a> {
    DoubleEntry(double_entry::DoubleEntry<'a>),
    DuplicateTransaction(duplicates::DuplicateTransaction<'a>),
    UnbalancedEntry(unbalanced::UnbalancedEntry<'a>),
    NonSequentialAppendix(sequential_appendix::NonSequentialAppendix<'a>),
    DuplicateAppendix(duplicate_appendix::DuplicateAppendix<'a>),
    MissingAppendix(appendix_missing::MissingAppendix<'a>),
}

impl<'a> Display for Lint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lint::DoubleEntry(inner) => write!(f, "{}", inner),
            Lint::DuplicateTransaction(inner) => write!(f, "{}", inner),
            Lint::UnbalancedEntry(inner) => write!(f, "{}", inner),
            Lint::NonSequentialAppendix(inner) => write!(f, "{}", inner),
            Lint::DuplicateAppendix(inner) => write!(f, "{}", inner),
            Lint::MissingAppendix(inner) => write!(f, "{}", inner),
        }
    }
}
