use beancount_core::{metadata::MetaValue, Transaction};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    appendix::{Appendix, AppendixError, AppendixExtractionError, AppendixExtractor},
    ledger::Sourced,
};

pub struct FromStatementPath;

// Matches 2000-01-01.{AppendixID}.*
static DATE_DOT_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(".*/?\\d\\d\\d\\d\\-\\d\\d\\-\\d\\d\\.(\\d+)\\..*").unwrap());

impl<'a> AppendixExtractor<'a> for FromStatementPath {
    fn from_transaction(
        transaction: Sourced<'a, Transaction<'a>>,
    ) -> Result<Appendix, AppendixError> {
        let statement = transaction
            .meta
            .get("statement")
            .ok_or(AppendixError::NotFound)?;

        let statement = match statement {
            MetaValue::Text(statement) => Ok(statement),
            _ => Err(AppendixExtractionError::StatementWrongType),
        }?;

        let captures = DATE_DOT_ID
            .captures(statement)
            .ok_or(AppendixExtractionError::CaptureMatchFailed)?;

        let stringified_id = captures
            .get(1)
            .ok_or(AppendixExtractionError::NoCaptures)?
            .as_str();

        let id: u64 = stringified_id
            .parse()
            .map_err(|_| AppendixExtractionError::ConversionFailed)?;

        Ok(Appendix {
            statement: statement.to_string(),
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use beancount_core::Transaction;

    use crate::{
        appendix::{
            statement::{AppendixExtractionError, FromStatementPath},
            Appendix, AppendixError, AppendixExtractor,
        },
        inline_ledger,
        ledger::Downcast,
    };

    #[test]
    fn test_extract_appendix_id_from_path() {
        let ledger = inline_ledger!(
            r#"
        2000-01-01 * "Example Payee" ""
            statement: "documents/hello/2022-01-01.1337.my-document.pdf"
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings
        "#
        );

        let appendix = ledger
            .directives()
            .into_iter()
            .filter_map(Transaction::downcast)
            .map(FromStatementPath::from_transaction)
            .next()
            .unwrap()
            .unwrap();

        assert_eq!(
            appendix,
            Appendix {
                statement: "documents/hello/2022-01-01.1337.my-document.pdf".to_string(),
                id: 1337
            }
        )
    }

    #[test]
    fn test_capture_match_failed() {
        let ledger = inline_ledger!(
            r#"
        2000-01-01 * "Example Payee" ""
            statement: "documents/hello/2022-01-01.HELLO.my-document.pdf"
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings
        "#
        );

        let appendix = ledger
            .directives()
            .into_iter()
            .filter_map(Transaction::downcast)
            .map(FromStatementPath::from_transaction)
            .next()
            .unwrap();

        assert_eq!(
            appendix,
            Err(AppendixError::ExtractionError(
                AppendixExtractionError::CaptureMatchFailed
            ))
        );
    }

    #[test]
    fn test_conversion_failure() {
        let ledger = inline_ledger!(
            r#"
        2000-01-01 * "Example Payee" ""
            statement: "documents/hello/2022-01-01.10000000000000000000000000000000000000000000000000.my-document.pdf"
            Assets:Bank:Account  -1500 DKK
            Assets:Bank:Savings
        "#
        );

        let appendix = ledger
            .directives()
            .into_iter()
            .filter_map(Transaction::downcast)
            .map(FromStatementPath::from_transaction)
            .next()
            .unwrap();

        assert_eq!(
            appendix,
            Err(AppendixError::ExtractionError(
                AppendixExtractionError::ConversionFailed
            ))
        );
    }
}
