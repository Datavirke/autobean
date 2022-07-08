use std::fmt::Display;

use beancount_core::{Directive, Transaction};
use colored::Colorize;
use log::debug;

use crate::{
    appendix::AppendixExtractor,
    ledger::{Downcast, Sourced},
    readable::Payees,
};

use super::Lint;

#[derive(Debug, PartialEq, Eq)]
pub struct MissingAppendix<'a> {
    entry: Sourced<'a, Transaction<'a>>,
}

impl<'a> From<Sourced<'a, Transaction<'a>>> for MissingAppendix<'a> {
    fn from(entry: Sourced<'a, Transaction<'a>>) -> Self {
        MissingAppendix { entry }
    }
}

impl<'a> From<MissingAppendix<'a>> for Lint<'a> {
    fn from(duplicate_appendix: MissingAppendix<'a>) -> Self {
        Lint::MissingAppendix(duplicate_appendix)
    }
}

impl<'a> Display for MissingAppendix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} transaction {} does not have an appendix attached:",
            "warning:".yellow().bold(),
            Payees::from(&self.entry)
        )?;

        writeln!(f, "{}", self.entry.location)
    }
}

pub fn find_missing_appendices<'a, Extractor: AppendixExtractor<'a>>(
    directives: &[Sourced<'a, Directive<'a>>],
) -> Vec<Lint<'a>> {
    debug!("checking for missing appendices");
    directives
        .iter()
        .cloned()
        .filter_map(Transaction::downcast)
        .filter(|txn| !txn.meta.contains_key("statement"))
        .map(MissingAppendix::from)
        .map(Lint::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        appendix::statement::FromStatementPath, inline_ledger,
        lints::appendix_missing::find_missing_appendices,
    };

    #[test]
    fn test_missing_appendices() {
        let ledger = inline_ledger!(
            r#"
        2000-01-04 * "Invoice" ""
            statement: "documents/2022-01-04.3.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-03 * "Invoice" ""
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-02 * "Invoice" ""
            statement: "documents/2022-01-02.2.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        
        2000-01-01 * "Invoice" ""
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        "#
        );

        let missing_appendices = find_missing_appendices::<FromStatementPath>(&ledger.directives());
        assert_eq!(missing_appendices.len(), 2);

        for missing in missing_appendices {
            println!("{}", missing);
        }
    }
}
