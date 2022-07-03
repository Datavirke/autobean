use std::{collections::HashMap, fmt::Display};

use beancount_core::{Directive, Transaction};
use colored::Colorize;

use crate::{
    appendix::{AppendixExtractor, TransactionWithAppendix},
    ledger::{Downcast, Sourced},
    location::ToLocationSpan,
};

use super::Lint;

#[derive(Debug, PartialEq, Eq)]
pub struct NonSequentialAppendix<'a> {
    before: TransactionWithAppendix<'a>,
    after: TransactionWithAppendix<'a>,
}

impl<'a> From<NonSequentialAppendix<'a>> for Lint<'a> {
    fn from(nonsequential_appdendix: NonSequentialAppendix<'a>) -> Self {
        Lint::NonSequentialAppendix(nonsequential_appdendix)
    }
}

impl<'a> Display for NonSequentialAppendix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} nonsequential appendix ids between {} and {} ({} --> {}):",
            "warning:".yellow().bold(),
            self.before
                .transaction
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
            self.after
                .transaction
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
            self.before.appendix.id,
            self.after.appendix.id
        )?;

        for source in [
            self.before.transaction.location.clone(),
            self.after.transaction.location.clone(),
        ]
        .into_iter()
        .to_span(10)
        {
            writeln!(f, "{}", source)?;
        }

        Ok(())
    }
}

pub fn find_nonsequential_appendices<'a, Extractor: AppendixExtractor<'a>>(
    directives: &[Sourced<'a, Directive<'a>>],
) -> Vec<Lint<'a>> {
    let appendices: HashMap<u64, _> = directives
        .iter()
        .cloned()
        .filter_map(Transaction::downcast)
        .filter_map(|transaction| {
            // For the sake of brevity, in this check we're ignoring transactions
            // that don't contain, or contain an unparseable appendix id.
            if let Ok(appendix) = Extractor::from_transaction(transaction.clone()) {
                Some((
                    appendix.id,
                    TransactionWithAppendix {
                        transaction,
                        appendix,
                    },
                ))
            } else {
                None
            }
        })
        .collect();

    let mut keys: Vec<u64> = appendices.keys().cloned().collect();
    keys.sort();

    keys.iter()
        .copied()
        .fold(
            (keys.first().copied().unwrap_or(0) - 1, vec![]),
            |(latest, mut gaps), key| {
                if key != latest + 1 {
                    gaps.push(
                        NonSequentialAppendix {
                            before: appendices.get(&latest).unwrap().clone(),
                            after: appendices.get(&key).unwrap().clone(),
                        }
                        .into(),
                    )
                }

                (key, gaps)
            },
        )
        .1
}

#[cfg(test)]
mod tests {
    use crate::{appendix::statement::FromStatementPath, inline_ledger};

    use super::find_nonsequential_appendices;

    #[test]
    fn test_nonsequential_appendix() {
        let ledger = inline_ledger!(
            r#"
        2000-01-04 * "Invoice" ""
            statement: "documents/2022-01-04.5.invoice.pdf"
            Assets:Bank:Account  -1500 DKK
            Expenses:Utilities:Power

        2000-01-03 * "Invoice" ""
            statement: "documents/2022-01-03.4.invoice.pdf"
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
            find_nonsequential_appendices::<FromStatementPath>(&ledger.directives());
        assert_eq!(nonsequential_appendices.len(), 1);

        let gap = nonsequential_appendices.first().unwrap();
        println!("{}", gap);
    }
}
