mod annual;
mod appendix;
mod balance;
mod error;
mod ledger;
mod lints;
mod location;
mod readable;

use std::{collections::HashMap, path::PathBuf, process::exit};

use appendix::statement::FromStatementPath;
use beancount_core::Transaction;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use itertools::Itertools;
use ledger::Ledger;
use log::{debug, warn, LevelFilter};
use tabled::{settings::Style, Table};

use crate::{
    appendix::{Appendix, AppendixExtractor},
    balance::balance,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
enum TableStyle {
    #[default]
    Blank,
    Ascii,
    Modern,
    Dots,
    Markdown,
    Psql,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Check ledger for all lints.
    Check,
    /// List all appendices listed in the ledger.
    ListAppendices,
    /// Produce a complete accounting of the given year.
    AnnualAccounts {
        /// Year whose transactions are to be considered.
        #[arg(long, short)]
        year: usize,
        /// Style of the output table
        #[arg(long, short, value_enum, default_value_t = TableStyle::Blank)]
        style: TableStyle,
    },
    /// Generate a list of balances, optionally up to and
    /// including a given year. If not provided, returns
    /// the current balance.
    Balance {
        /// Include all transactions up to and including this year
        #[arg(long, short = 'y')]
        up_to_and_including: Option<usize>,
        /// Style of the output table
        #[arg(long, short, value_enum, default_value_t = TableStyle::Blank)]
        style: TableStyle,
    },
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
        Commands::Balance {
            up_to_and_including,
            style,
        } => {
            let table = apply_style(balance::balance(&ledger, up_to_and_including), style);

            println!("{}", table);
        }
        Commands::AnnualAccounts { year, style } => {
            let (table, statements) = annual::accounts(&ledger, year);
            let table = apply_style(table, style);

            let output_path = PathBuf::from(year.to_string());
            std::fs::remove_dir_all(&output_path).ok();
            std::fs::create_dir_all(&output_path.join("bilag")).unwrap();
            for statement in statements {
                let destination = output_path
                    .join("bilag")
                    .join(statement.file_name().unwrap());
                std::fs::copy(&statement, destination).unwrap();
            }

            let initial_balance = apply_style(balance(&ledger, Some(year)), style);
            let final_balance = apply_style(balance(&ledger, Some(year)), style);

            std::fs::write(
                output_path.join("startsaldo.txt"),
                initial_balance.to_string(),
            )
            .unwrap();

            std::fs::write(output_path.join("slutsaldo.txt"), final_balance.to_string()).unwrap();

            std::fs::write(output_path.join("poster.txt"), table.to_string()).unwrap();
        }
    }
}

fn apply_style(mut table: Table, style: TableStyle) -> Table {
    match style {
        TableStyle::Blank => table.with(Style::blank()),
        TableStyle::Ascii => table.with(Style::ascii()),
        TableStyle::Modern => table.with(Style::modern()),
        TableStyle::Dots => table.with(Style::dots()),
        TableStyle::Markdown => table.with(Style::markdown()),
        TableStyle::Psql => table.with(Style::psql()),
    };
    table
}
