use std::path::PathBuf;

use beancount_parser::error::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("loading ledger {0}, {1}")]
    Ledger(PathBuf, ParseError),
}
