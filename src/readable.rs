use std::fmt::Display;

use beancount_core::Transaction;
use colored::Colorize;

use crate::ledger::Sourced;

pub struct Payees(Vec<String>);

impl<'a> From<&Transaction<'a>> for Payees {
    fn from(transaction: &Transaction<'a>) -> Self {
        Payees(if let Some(payee) = transaction.payee.as_deref() {
            vec![payee.to_string()]
        } else {
            vec![]
        })
    }
}

impl<'a> From<&Sourced<'a, Transaction<'a>>> for Payees {
    fn from(transaction: &Sourced<'a, Transaction<'a>>) -> Self {
        Payees::from(&transaction.inner)
    }
}

impl<'a, T: Into<Transaction<'a>> + Clone> From<&Vec<T>> for Payees {
    fn from(vec: &Vec<T>) -> Self {
        Payees(
            vec.iter()
                .cloned()
                .filter_map(|txn| txn.into().payee.as_deref().map(String::from))
                .collect(),
        )
    }
}

impl<'a, T: Into<Transaction<'a>>> From<Vec<T>> for Payees {
    fn from(vec: Vec<T>) -> Self {
        Payees(
            vec.into_iter()
                .filter_map(|txn| txn.into().payee.as_deref().map(String::from))
                .collect(),
        )
    }
}

impl<'a, T> From<&[T]> for Payees
where
    T: Into<Transaction<'a>> + Clone,
{
    fn from(vec: &[T]) -> Self {
        Payees(
            vec.iter()
                .cloned()
                .filter_map(|txn| txn.into().payee.as_deref().map(String::from))
                .collect(),
        )
    }
}

impl<'a, const N: usize, T> From<&[T; N]> for Payees
where
    T: Into<Transaction<'a>> + Clone,
{
    fn from(vec: &[T; N]) -> Self {
        Payees(
            vec.iter()
                .cloned()
                .filter_map(|txn| txn.into().payee.as_deref().map(String::from))
                .collect(),
        )
    }
}

impl Display for Payees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, n) in self.0.iter().enumerate() {
            if self.0.len() > 1 && idx + 1 == self.0.len() {
                write!(f, " and {}", n.bold().green())?;
            } else if self.0.len() > 2 {
                write!(f, "{},", n.bold().green())?;
            } else {
                write!(f, "{}", n.bold().green())?;
            }
        }

        Ok(())
    }
}
