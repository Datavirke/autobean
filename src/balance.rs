use std::{borrow::Cow, collections::HashMap};

use beancount_core::{metadata::MetaValue, Account, AccountType, Date, Directive};
use rust_decimal::Decimal;
use tabled::{
    settings::{object::Columns, Alignment},
    Table, Tabled,
};

use crate::ledger::Ledger;

#[derive(Debug)]
pub struct Transfer<'a> {
    pub account: Account<'a>,
    pub amount: Decimal,
}

#[derive(Debug)]
pub struct Item<'a> {
    pub date: Date<'a>,
    pub statement: Cow<'a, str>,
    pub payee: Option<Cow<'a, str>>,
    pub description: Cow<'a, str>,
    pub postings: Vec<Transfer<'a>>,
}

fn sums_for<'a>(items: impl Iterator<Item = &'a Item<'a>>) -> HashMap<Account<'a>, Decimal> {
    let mut sums = HashMap::<Account, Decimal>::default();

    for item in items {
        for posting in item.postings.iter() {
            sums.entry(posting.account.clone())
                .and_modify(|amount| *amount += posting.amount)
                .or_insert(posting.amount);
        }
    }

    sums
}

pub fn itemized_transactions(ledger: &Ledger) -> Vec<Item> {
    let mut items = Vec::<Item>::default();

    for directive in ledger.directives() {
        match directive.inner {
            Directive::Transaction(mut txn) => {
                let Some(MetaValue::Text(statement)) = txn.meta.remove("statement") else {
                    panic!(
                        "Transaction {:#?} does not contain a statement!",
                        directive.location
                    );
                };

                let mut item = Item {
                    date: txn.date,
                    statement,
                    payee: txn.payee,
                    description: txn.narration,
                    postings: Vec::new(),
                };

                let mut postings = txn.postings.clone();
                postings.sort_by_key(|p| p.units.num);

                loop {
                    let Some(posting) = postings.pop() else { break };

                    if let Some(num) = posting.units.num {
                        let num = if posting
                            .units
                            .currency
                            .as_ref()
                            .is_some_and(|currency| currency != "DKK")
                        {
                            if let Some(price) = posting.price.and_then(|price| price.num) {
                                num * price
                            } else {
                                panic!("No price attached!")
                            }
                        } else {
                            num
                        };

                        item.postings.push(Transfer {
                            account: posting.account,
                            amount: num,
                        });
                    } else {
                        assert!(postings.is_empty(), "cannot compute remaining amount if there's more than one posting left.");

                        let total: Decimal =
                            item.postings.iter().map(|posting| posting.amount).sum();

                        item.postings.push(Transfer {
                            account: posting.account,
                            amount: -total,
                        })
                    }
                }

                items.push(item);
            }

            _ => (),
        }
    }

    items.sort_by_key(|item| item.date.to_string());
    items
}

pub fn balance(ledger: &Ledger, up_to_and_including: Option<usize>) -> Table {
    let items = itemized_transactions(&ledger);

    let balance = if let Some(year) = up_to_and_including {
        let end_date = Date::from_string_unchecked(format!("{year}-01-01", year = year + 1));
        sums_for(items.iter().filter(|item| item.date < end_date))
    } else {
        sums_for(items.iter())
    };

    tabled_balance(balance.iter())
}

fn tabled_balance<'a>(balances: impl Iterator<Item = (&'a Account<'a>, &'a Decimal)>) -> Table {
    let mut table = Table::new(
        balances
            .filter(|item| !item.1.round_dp(2).is_zero())
            .filter(|item| item.0.ty != AccountType::Income)
            .filter(|item| item.0.ty != AccountType::Expenses)
            .map(Balance),
    );

    table.modify(Columns::last(), Alignment::right());
    table
}

#[derive(PartialEq, Eq)]
pub struct Balance<'a>(pub (&'a Account<'a>, &'a Decimal));

impl<'a> Balance<'a> {
    pub fn name(&self) -> String {
        format!("{:?}:{}", self.0 .0.ty, self.0 .0.parts.join(":"),)
    }

    pub fn balance(&self) -> Decimal {
        self.0 .1.clone()
    }
}

impl<'a> Tabled for Balance<'a> {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name().into(),
            self.balance().round_dp(2).to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["Account".into(), "Balance".into()]
    }
}

impl<'a> Ord for Balance<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(&other.name())
    }
}

impl<'a> PartialOrd for Balance<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
