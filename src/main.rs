mod error;
mod fingerprint;
mod ledger;
mod location;

use beancount_core::Directive;
use colored::Colorize;
use fingerprint::Fingerprint;
use ledger::{Ledger, Sourced};
use location::{Location, ToLocationSpan};
use std::collections::HashMap;

fn main() {
    let ledger = Ledger::from_path("data").unwrap();

    let directives = ledger.directives();
    find_duplicates(&directives);
}

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
