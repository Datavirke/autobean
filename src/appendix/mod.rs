pub mod statement;

use beancount_core::{Directive, Transaction};
use thiserror::Error;

use crate::ledger::{Downcast, Sourced};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appendix {
    pub statement: String,
    pub id: u64,
}

pub trait AppendixExtractor<'a> {
    fn from_transaction(
        transaction: Sourced<'a, Transaction<'a>>,
    ) -> Result<Appendix, AppendixError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppendixError {
    #[error("no field identifying an associated appendix was found within the transaction")]
    NotFound,
    #[error(
        "an error occurred while attempting to extract the appendix from the transaction: {0}"
    )]
    ExtractionError(#[from] AppendixExtractionError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppendixExtractionError {
    #[error("statement is not a string")]
    StatementWrongType,
    #[error("capture expression did not match statement")]
    CaptureMatchFailed,
    #[error("capture expression matched, but no capture groups extracted")]
    NoCaptures,
    #[error("the statement id could not be converted to a 64-bit unsigned integer")]
    ConversionFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionWithAppendix<'a> {
    pub transaction: Sourced<'a, Transaction<'a>>,
    pub appendix: Appendix,
}

impl<'a> From<TransactionWithAppendix<'a>> for Transaction<'a> {
    fn from(txn: TransactionWithAppendix<'a>) -> Self {
        txn.transaction.inner
    }
}

pub trait IntoAppendices<'a> {
    fn into_appendices<Extractor: AppendixExtractor<'a>>(self) -> Vec<TransactionWithAppendix<'a>>;
}

impl<'a, I: Iterator<Item = Sourced<'a, Directive<'a>>>> IntoAppendices<'a> for I {
    fn into_appendices<Extractor: AppendixExtractor<'a>>(self) -> Vec<TransactionWithAppendix<'a>> {
        self.filter_map(Transaction::downcast)
            .filter_map(|transaction| {
                // For the sake of brevity, in this check we're ignoring transactions
                // that don't contain, or contain an unparseable appendix id.
                if let Ok(appendix) = Extractor::from_transaction(transaction.clone()) {
                    Some(TransactionWithAppendix {
                        transaction,
                        appendix,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
