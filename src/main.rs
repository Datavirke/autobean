mod appendix;
mod error;
mod ledger;
mod lints;
mod location;
mod readable;

use std::process::exit;

use appendix::statement::FromStatementPath;
use clap::Parser;
use ledger::Ledger;
use log::{debug, LevelFilter, warn};

/// Lints beancount files in a directory
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path in which to look for *.beancount files.
    /// Defaults to working directory.
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String,

    /// Debug level for the application logger. One of:
    /// off, error, warn, info, debug or trace
    #[clap(short, long, value_parser, default_value_t = LevelFilter::Off)]
    debug: LevelFilter,
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

    let lints: Vec<_> = [
        lints::find_double_entries(&directives),
        lints::find_duplicates(&directives),
        lints::find_unbalanced_entries(&directives),
        lints::find_nonsequential_appendices::<FromStatementPath>(&directives),
        lints::find_duplicate_appendix_ids::<FromStatementPath>(&directives),
        lints::find_missing_appendices::<FromStatementPath>(&directives),
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
