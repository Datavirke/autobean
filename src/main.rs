mod appendix;
mod error;
mod ledger;
mod lints;
mod location;
mod readable;

use std::{collections::HashMap, process::exit};

use appendix::statement::FromStatementPath;
use beancount_core::Transaction;
use clap::{Parser, Subcommand};
use colored::Colorize;
use itertools::Itertools;
use ledger::Ledger;
use log::{debug, warn, LevelFilter};

use crate::{
    appendix::{Appendix, AppendixExtractor},
    ledger::Downcast,
};

/// Lints beancount files in a directory
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path in which to look for *.beancount files.
    /// Defaults to working directory.
    #[arg(default_value_t = String::from("."))]
    path: String,

    /// Debug level for the application logger. One of:
    /// off, error, warn, info, debug or trace
    #[arg(short, long, default_value_t = LevelFilter::Off)]
    debug: LevelFilter,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Check ledger for all lints.
    Check,
    /// List all appendices listed in the ledger.
    ListAppendices,
}

fn main() {
    let args = Args::parse();

    pretty_env_logger::formatted_timed_builder()
        .filter_level(args.debug)
        .init();

    let ledger = Ledger::from_path(&args.path).unwrap();
    debug!("loading ledgers from: {}", &args.path);

    let directives = ledger.directives();
    if directives.is_empty() {
        warn!("ledger contains no directives, are you sure the directory contains any beancount files?");
    } else {
        debug!("compiled ledger contains {} directives", directives.len());
    }

    match args.command {
        Commands::Check => {
            let lints: Vec<_> = [
                lints::find_double_entries(&directives),
                lints::find_duplicates(&directives),
                lints::find_unbalanced_entries(&directives),
                lints::find_nonsequential_appendices::<FromStatementPath>(&directives),
                lints::find_duplicate_appendix_ids::<FromStatementPath>(&directives),
                lints::find_missing_appendices::<FromStatementPath>(&directives),
                lints::find_missing_documents::<FromStatementPath>(&directives),
            ]
            .into_iter()
            .flatten()
            .collect();

            debug!("discovered {} issues", lints.len());

            for lint in &lints {
                eprint!("{}", lint);
            }

            if !lints.is_empty() {
                exit(1);
            }

            exit(0)
        }
        Commands::ListAppendices => {
            // Use a HashSet to uniquely identify each appendix
            let appendices: HashMap<Appendix, Vec<_>> = directives
                .iter()
                .cloned()
                .filter_map(Transaction::downcast)
                .filter_map(|transaction| {
                    if let Ok(appendix) = FromStatementPath::from_transaction(transaction.clone()) {
                        Some((appendix, transaction))
                    } else {
                        None
                    }
                })
                .into_group_map();

            // Convert to Vec, since HashSets by design are unordered.
            let mut appendices: Vec<_> = appendices.into_iter().collect();
            appendices.sort_by(|(a, _), (b, _)| a.cmp(b));

            for (Appendix { id, statement }, transactions) in appendices {
                println!("{id: >8} {statement}", statement = statement.bold().green());
                for transaction in transactions {
                    println!(
                        "       ↳ {ledger}:{line}",
                        ledger = transaction
                            .location
                            .ledger()
                            .source
                            .filename()
                            .to_string_lossy(),
                        line = transaction.location.start()
                    );
                }
                println!("");
            }
        }
    }
}
