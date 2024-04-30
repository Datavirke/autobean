use std::path::PathBuf;

use beancount_core::Date;
use tabled::{
    settings::{object::Cell, Alignment},
    Table,
};

use crate::{
    balance::{self, Balance},
    ledger::Ledger,
};

pub fn accounts(ledger: &Ledger, year: usize) -> (Table, Vec<PathBuf>) {
    let items = balance::itemized_transactions(&ledger);

    let mut statements = Vec::new();

    let mut table = tabled::builder::Builder::new();

    let start = Date::from_string_unchecked(format!("{year}-01-01"));
    let end = Date::from_string_unchecked(format!("{year}-01-01", year = year + 1));

    table.push_record(["Dato", "Beskrivelse/Konto/Sti", "Beløb"]);
    let mut cells = vec![Cell::new(0, 2)];
    let mut current_row = 1;
    for item in items {
        if item.date < start || item.date >= end {
            continue;
        }

        table.push_record([
            item.date.to_string(),
            format!(
                "{}: {}",
                item.payee.map(String::from).unwrap(),
                item.description.to_string()
            ),
        ]);

        current_row += 1;

        for posting in item.postings {
            let balance = Balance((&posting.account, &posting.amount));
            table.push_record([
                "".to_string(),
                balance.name(),
                format!("{} DKK", balance.balance()),
            ]);

            cells.push(Cell::new(current_row, 2));
            current_row += 1
        }

        let statement = PathBuf::from(item.statement.as_ref());

        table.push_record([
            "Bilag".to_string(),
            statement.file_name().unwrap().to_string_lossy().to_string(),
            "".to_string(),
        ]);
        statements.push(statement);
        current_row += 1;

        table.push_record(["", "", ""]);
        current_row += 1;
    }

    table.remove_record(current_row - 1);
    let mut table = table.build();
    for cell in cells {
        table.modify(cell, Alignment::right());
    }

    (table, statements)
}
