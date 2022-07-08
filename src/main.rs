mod appendix;
mod error;
mod ledger;
mod lints;
mod location;
mod readable;

use appendix::statement::FromStatementPath;
use clap::Parser;
use ledger::Ledger;

/// Lints beancount files in a directory
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path in which to look for *.beancount files.
    /// Defaults to working directory.
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String,
}

fn main() {
    let args = Args::parse();

    let ledger = Ledger::from_path(args.path).unwrap();

    let directives = ledger.directives();

    for lint in [
        lints::find_double_entries(&directives),
        lints::find_duplicates(&directives),
        lints::find_unbalanced_entries(&directives),
        lints::find_nonsequential_appendices::<FromStatementPath>(&directives),
        lints::find_duplicate_appendix_ids::<FromStatementPath>(&directives),
        lints::find_missing_appendices::<FromStatementPath>(&directives),
    ]
    .iter()
    .flatten()
    {
        print!("{}", lint);
    }
}
