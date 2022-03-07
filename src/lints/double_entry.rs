use std::{collections::HashSet, fmt::Display};

use beancount_core::{Directive, Transaction};
use colored::Colorize;

use crate::{ledger::Sourced, lints::Lint, location::ToLocationSpan};

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct DoubleEntry<'a> {
    entries: [Sourced<'a, Transaction<'a>>; 2],
}

impl<'a> DoubleEntry<'a> {
    pub fn from(entries: &[Sourced<'a, Transaction<'a>>]) -> Self {
        let mut entries: [Sourced<'a, Transaction<'a>>; 2] =
            [entries[0].clone(), entries[1].clone()];

        entries.sort_by(|a, b| a.location.cmp(&b.location));

        DoubleEntry { entries }
    }
}

impl<'a> From<DoubleEntry<'a>> for Lint<'a> {
    fn from(double_entry: DoubleEntry<'a>) -> Self {
        Lint::DoubleEntry(double_entry)
    }
}

impl<'a> Display for DoubleEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} potential double entry transaction {} and {}:",
            "warning:".yellow().bold(),
            self.entries[0]
                .inner
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
            self.entries[1]
                .inner
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green()
        )?;

        for source in [
            self.entries[0].location.clone(),
            self.entries[1].location.clone(),
        ]
        .into_iter()
        .to_span(10)
        {
            writeln!(f, "{}", source)?;
        }

        Ok(())
    }
}

pub fn find_double_entries<'a>(directives: &[Sourced<'a, Directive<'a>>]) -> Vec<Lint<'a>> {
    let mut double_entries = HashSet::new();

    for first in directives.iter() {
        if let Directive::Transaction(a) = &first.inner {
            for second in directives.iter() {
                if let Directive::Transaction(b) = &second.inner {
                    // If the dates aren't the same, then it can't be the same transaction (probably)
                    if a.date != b.date {
                        continue;
                    }

                    // Check if the source of Txn A is listed in Txn B (but not the first)
                    let a_source = if let Some(a_source) = a.postings.first() {
                        if !b
                            .postings
                            .iter()
                            .skip(1)
                            .any(|p| p.account == a_source.account)
                        {
                            continue;
                        }

                        a_source
                    } else {
                        continue;
                    };

                    // And vice versa
                    let b_source = if let Some(b_source) = b.postings.first() {
                        if !a
                            .postings
                            .iter()
                            .skip(1)
                            .any(|p| p.account == b_source.account)
                        {
                            continue;
                        }

                        b_source
                    } else {
                        continue;
                    };

                    // Should also check if accounts are Assets.

                    if a_source.units.currency != b_source.units.currency {
                        continue;
                    }

                    if let (Some(a_num), Some(b_num)) = (a_source.units.num, b_source.units.num) {
                        if (a_num + b_num).is_zero() {
                            double_entries.insert(DoubleEntry::from(&[
                                Sourced {
                                    inner: a.clone(),
                                    location: first.location.clone(),
                                },
                                Sourced {
                                    inner: b.clone(),
                                    location: second.location.clone(),
                                },
                            ]));
                        }
                    }
                }
            }
        }
    }

    double_entries.into_iter().map(Lint::from).collect()
}

#[cfg(test)]
mod tests {
    use super::find_double_entries;
    use crate::inline_ledger;

    #[test]
    fn test_double_entry() {
        let ledger = inline_ledger!(
            r#"
        2000-01-01 * "Example Payee" ""
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings

        2000-01-01 * "Example Payee" ""
            Assets:Bank:Savings  1500 DKK
            Assets:Bank:Account

        2000-01-01 * "Unrelated Transaction" ""
            Assets:Bank:Savings  1 DKK
            Assets:Bank:Account
        "#
        );

        let double_entries = find_double_entries(&ledger.directives());
        assert!(double_entries.len() == 1);
    }
}
