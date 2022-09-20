use std::{fmt::Display, path::PathBuf};

use beancount_core::{metadata::MetaValue, Directive, Transaction};
use colored::Colorize;
use log::debug;

use crate::{
    appendix::AppendixExtractor,
    ledger::{Downcast, Sourced},
    readable::Payees,
};

use super::Lint;

#[derive(Debug, PartialEq, Eq)]
pub struct MissingDocument<'a> {
    entry: Sourced<'a, Transaction<'a>>,
}

impl<'a> From<Sourced<'a, Transaction<'a>>> for MissingDocument<'a> {
    fn from(entry: Sourced<'a, Transaction<'a>>) -> Self {
        MissingDocument { entry }
    }
}

impl<'a> From<MissingDocument<'a>> for Lint<'a> {
    fn from(missing_document: MissingDocument<'a>) -> Self {
        Lint::MissingDocument(missing_document)
    }
}

impl<'a> Display for MissingDocument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} transaction {}'s statement points to non-existent file path",
            "warning:".yellow().bold(),
            Payees::from(&self.entry)
        )?;

        writeln!(f, "{}", self.entry.location)
    }
}

pub fn find_missing_documents<'a, Extractor: AppendixExtractor<'a>>(
    directives: &[Sourced<'a, Directive<'a>>],
) -> Vec<Lint<'a>> {
    debug!("checking for missing documents");
    directives
        .iter()
        .cloned()
        .filter_map(Transaction::downcast)
        .filter(|txn| {
            if let Some(MetaValue::Text(path)) = txn.meta.get("statement") {
                !PathBuf::from(path.as_ref()).exists()
            } else {
                // If the transaction doesn't contain a statement, we can't verify it.
                // This kind of error should be handled by [`crate::lints::appendix_missing`]
                false
            }
        })
        .map(MissingDocument::from)
        .map(Lint::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        appendix::statement::FromStatementPath, inline_ledger,
        lints::document_missing::find_missing_documents,
    };

    #[test]
    fn test_missing_documents() {
        // We'll just pretend the library source files are the statements.
        let ledger = inline_ledger!(
            r#"
        2000-01-04 * "File exists- ok" ""
            statement: "src/main.rs"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-03 * "No statement - no error" ""
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-02 * "File exists - ok" ""
            statement: "Cargo.toml"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        
        2000-01-01 * "File not found!" ""
            statement: "non-existent-file.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        "#
        );

        let missing_documents = find_missing_documents::<FromStatementPath>(&ledger.directives());
        assert_eq!(missing_documents.len(), 1);

        for missing in missing_documents {
            println!("{}", missing);
        }
    }
}
