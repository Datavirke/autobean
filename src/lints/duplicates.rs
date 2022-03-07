use std::{borrow::Cow, collections::HashMap, fmt::Display};

use beancount_core::{Account, Date, Directive, IncompleteAmount, Posting, Transaction};
use colored::Colorize;

use crate::{ledger::Sourced, location::ToLocationSpan};

use super::Lint;

#[derive(Debug, PartialEq, Eq, Hash)]
struct PostingFingerprint<'a> {
    pub account: &'a Account<'a>,
    pub units: &'a IncompleteAmount<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TransactionFingerprint<'a> {
    pub date: &'a Date<'a>,
    pub payee: &'a Option<Cow<'a, str>>,
    pub postings: Vec<PostingFingerprint<'a>>,
}

trait Fingerprint<'a> {
    type Fingerprint: std::hash::Hash;
    fn fingerprint(&'a self) -> Self::Fingerprint;
}

impl<'a> Fingerprint<'a> for Posting<'a> {
    type Fingerprint = PostingFingerprint<'a>;

    fn fingerprint(&'a self) -> Self::Fingerprint {
        PostingFingerprint {
            account: &self.account,
            units: &self.units,
        }
    }
}

impl<'a> Fingerprint<'a> for Transaction<'a> {
    type Fingerprint = TransactionFingerprint<'a>;

    fn fingerprint(&'a self) -> Self::Fingerprint {
        TransactionFingerprint {
            date: &self.date,
            payee: &self.payee,
            postings: self.postings.iter().map(Fingerprint::fingerprint).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct DuplicateTransaction<'a> {
    entries: [Sourced<'a, Transaction<'a>>; 2],
}

impl<'a> DuplicateTransaction<'a> {
    pub fn from(entries: &[Sourced<'a, Transaction<'a>>]) -> Self {
        let mut entries: [Sourced<'a, Transaction<'a>>; 2] =
            [entries[0].clone(), entries[1].clone()];

        entries.sort_by(|a, b| a.location.cmp(&b.location));

        DuplicateTransaction { entries }
    }
}

impl<'a> From<DuplicateTransaction<'a>> for Lint<'a> {
    fn from(duplicate: DuplicateTransaction<'a>) -> Self {
        Lint::DuplicateTransaction(duplicate)
    }
}

impl<'a> Display for DuplicateTransaction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} identical transaction {} found in multiple locations:",
            "warning:".yellow().bold(),
            self.entries[0]
                .inner
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green(),
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

pub fn find_duplicates<'a>(directives: &[Sourced<'a, Directive<'a>>]) -> Vec<Lint<'a>> {
    let mut set = HashMap::new();

    for dir in directives.iter() {
        if let Directive::Transaction(txn) = &dir.inner {
            set.entry(txn.fingerprint())
                .and_modify(|sources: &mut Vec<Sourced<'a, Transaction<'a>>>| {
                    sources.push(Sourced {
                        location: dir.location.clone(),
                        inner: txn.clone(),
                    })
                })
                .or_insert_with(|| {
                    vec![Sourced {
                        location: dir.location.clone(),
                        inner: txn.clone(),
                    }]
                });
        }
    }

    set.values()
        .into_iter()
        .filter_map(|values| {
            if values.len() == 1 {
                None
            } else {
                Some(DuplicateTransaction::from(values.as_slice()))
            }
        })
        .map(Lint::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::find_duplicates;
    use crate::inline_ledger;

    #[test]
    fn test_duplicates() {
        let ledger = inline_ledger!(
            r#"
        2000-01-01 * "Example Payee" ""
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings

        2000-01-01 * "Example Payee" ""
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings

        2000-01-01 * "Example Payee" ""
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings
        "#
        );

        let duplicates = find_duplicates(&ledger.directives());
        assert_eq!(duplicates.len(), 1);
    }
}
