mod statement;

use beancount_core::Transaction;
use thiserror::Error;

use crate::ledger::Sourced;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appendix {
    statement: String,
    id: u64,
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
