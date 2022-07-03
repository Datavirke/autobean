use std::{collections::HashMap, fmt::Display};

use beancount_core::Directive;
use colored::Colorize;
use itertools::Itertools;

use crate::{
    appendix::{AppendixExtractor, IntoAppendices, TransactionWithAppendix},
    ledger::Sourced,
    location::ToLocationSpan,
};

use super::Lint;

#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateAppendix<'a> {
    entries: Vec<TransactionWithAppendix<'a>>,
}

impl<'a> From<DuplicateAppendix<'a>> for Lint<'a> {
    fn from(duplicate_appendix: DuplicateAppendix<'a>) -> Self {
        Lint::DuplicateAppendix(duplicate_appendix)
    }
}

impl<'a> Display for DuplicateAppendix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} appendix id {} is used in transactions {} and {}, but the appendices themselves are not the same.",
            "warning:".yellow().bold(),
            self.entries[0].appendix.id,
            self.entries[0]
                .transaction
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
            self.entries[1]
                .transaction
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
        )?;

        for source in [
            self.entries[0].transaction.location.clone(),
            self.entries[1].transaction.location.clone(),
        ]
        .into_iter()
        .to_span(10)
        {
            writeln!(f, "{}", source)?;
        }

        Ok(())
    }
}

pub fn find_duplicate_appendix_ids<'a, Extractor: AppendixExtractor<'a>>(
    directives: &[Sourced<'a, Directive<'a>>],
) -> Vec<Lint<'a>> {
    let appendices = directives.iter().cloned().into_appendices::<Extractor>();

    let duplicates: Vec<_> = appendices
        .iter()
        .group_by(|txn_appendix| txn_appendix.appendix.id)
        .into_iter()
        .filter_map(|(_, group)| {
            // Use a HashMap to filter out appendices which are identical.
            // It's perfectly legal to refer to the same appendix in multiple
            // transactions, but if the same ID is used across different statements
            // it's an error.
            let group: HashMap<_, _> = group
                .map(|txn| (&txn.appendix.statement, txn.clone()))
                .collect();
            if group.len() > 1 {
                Some(group.into_values().collect())
            } else {
                None
            }
        })
        .map(|entries| DuplicateAppendix { entries }.into())
        .collect();

    duplicates
}

#[cfg(test)]
mod tests {
    use crate::{
        appendix::statement::FromStatementPath, inline_ledger,
        lints::duplicate_appendix::find_duplicate_appendix_ids,
    };

    #[test]
    fn test_duplicate_appendix_ids() {
        let ledger = inline_ledger!(
            r#"
        2000-01-04 * "Invoice" ""
            statement: "documents/2022-01-04.3.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-03 * "Invoice" ""
            statement: "documents/2022-01-03.2.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-02 * "Invoice" ""
            statement: "documents/2022-01-02.2.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        
        2000-01-01 * "Invoice" ""
            statement: "documents/2022-01-01.1.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power
        "#
        );

        let nonsequential_appendices =
            find_duplicate_appendix_ids::<FromStatementPath>(&ledger.directives());
        assert_eq!(nonsequential_appendices.len(), 1);

        let duplicate = nonsequential_appendices.first().unwrap();
        println!("{}", duplicate);
    }
}
