mod appendix;
mod error;
mod ledger;
mod lints;
mod location;
mod readable;

use appendix::statement::FromStatementPath;
use ledger::Ledger;

fn main() {
    let ledger = Ledger::from_path("data/personal").unwrap();

    let directives = ledger.directives();

    for lint in [
        lints::find_double_entries(&directives),
        lints::find_duplicates(&directives),
        lints::find_unbalanced_entries(&directives),
        lints::find_nonsequential_appendices::<FromStatementPath>(&directives),
        lints::find_duplicate_appendix_ids::<FromStatementPath>(&directives),
    ]
    .iter()
    .flatten()
    {
        print!("{}", lint);
    }
}
