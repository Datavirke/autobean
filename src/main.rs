mod error;
mod ledger;
mod lints;
mod location;

use ledger::Ledger;

fn main() {
    let ledger = Ledger::from_path("data").unwrap();

    let directives = ledger.directives();

    for lint in [
        lints::find_double_entries(&directives),
        lints::find_duplicates(&directives),
    ]
    .iter()
    .flatten()
    {
        println!("{}", lint);
    }
}
