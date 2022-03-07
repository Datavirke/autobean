use std::fmt::Display;

use beancount_core::{Directive, Transaction};
use colored::Colorize;

use crate::ledger::Sourced;

use super::Lint;

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct UnbalancedEntry<'a> {
    entry: Sourced<'a, Transaction<'a>>,
}

impl<'a> UnbalancedEntry<'a> {
    pub fn from(entry: Sourced<'a, Transaction<'a>>) -> Self {
        UnbalancedEntry { entry }
    }
}

impl<'a> From<UnbalancedEntry<'a>> for Lint<'a> {
    fn from(unbalanced_entry: UnbalancedEntry<'a>) -> Self {
        Lint::UnbalancedEntry(unbalanced_entry)
    }
}

impl<'a> Display for UnbalancedEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} unbalanced transaction {}:",
            "warning:".yellow().bold(),
            self.entry
                .inner
                .payee
                .as_deref()
                .unwrap_or_default()
                .bold()
                .green()
        )?;

        writeln!(f, "{}", self.entry.location)
    }
}

pub fn find_unbalanced_entries<'a>(directives: &[Sourced<'a, Directive<'a>>]) -> Vec<Lint<'a>> {
    directives
        .iter()
        .filter_map(|dir| {
            if let Directive::Transaction(txn) = &dir.inner {
                // TODO: Do some checks in case there are more than one posting,
                // but the total doesn't add up.
                if txn.postings.len() == 1 {
                    Some(
                        UnbalancedEntry::from(Sourced {
                            location: dir.location.clone(),
                            inner: txn.clone(),
                        })
                        .into(),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
