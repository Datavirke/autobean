mod error;
mod fingerprint;
mod ledger;
mod location;

use beancount_core::{Directive, Transaction};
use colored::Colorize;
use fingerprint::Fingerprint;
use ledger::{Ledger, Sourced};
use location::{Location, ToLocationSpan};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

fn main() {
    let ledger = Ledger::from_path("data").unwrap();

    let directives = ledger.directives();
    //find_duplicates(&directives);

    find_double_entries(&directives);
}

#[derive(PartialEq, Hash, Eq)]
pub struct DoubleEntryWarning<'a> {
    entries: [Sourced<'a, Transaction<'a>>; 2],
}

impl<'a> DoubleEntryWarning<'a> {
    pub fn from(entries: &[Sourced<'a, Transaction<'a>>]) -> Self {
        let mut entries: [Sourced<'a, Transaction<'a>>; 2] =
            [entries[0].clone(), entries[1].clone()];

        entries.sort_by(|a, b| a.location.cmp(&b.location));

        DoubleEntryWarning { entries }
    }
}

impl<'a> Display for DoubleEntryWarning<'a> {
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

pub enum Lint<'a> {
    DoubleEntryWarning(DoubleEntryWarning<'a>),
}

#[allow(dead_code)]
fn find_duplicates<'a>(directives: &[Sourced<'a, Directive<'a>>]) {
    let mut set = HashMap::new();

    for dir in directives.iter() {
        if let Directive::Transaction(txn) = &dir.inner {
            set.entry(txn.fingerprint())
                .and_modify(|sources: &mut Vec<Location>| sources.push(dir.location.clone()))
                .or_insert_with(|| vec![dir.location.clone()]);
        }
    }

    for (txn, sources) in set.into_iter().filter(|(_, sources)| sources.len() > 1) {
        println!(
            "{} identical transaction {} found in multiple locations:",
            "warning:".yellow().bold(),
            txn.payee.as_deref().unwrap_or_default().bold().green()
        );

        for source in sources.into_iter().to_span(10) {
            println!("{}", source)
        }
    }
}

fn find_double_entries<'a>(directives: &[Sourced<'a, Directive<'a>>]) {
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
                            double_entries.insert(DoubleEntryWarning::from(&[
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

    for double_entry in double_entries {
        println!("{}", double_entry);
    }
}
