use std::borrow::Cow;

use beancount_core::{Account, Date, IncompleteAmount, Posting, Transaction};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PostingFingerprint<'a> {
    pub account: &'a Account<'a>,
    pub units: &'a IncompleteAmount<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TransactionFingerprint<'a> {
    pub date: &'a Date<'a>,
    pub payee: &'a Option<Cow<'a, str>>,
    pub postings: Vec<PostingFingerprint<'a>>,
}

pub trait Fingerprint<'a> {
    type Fingerprint: std::hash::Hash;
    fn fingerprint(&'a self) -> Self::Fingerprint;
}

impl<'a> Fingerprint<'a> for Posting<'a> {
    type Fingerprint = PostingFingerprint<'a>;

    fn fingerprint(&'a self) -> Self::Fingerprint {
        PostingFingerprint {
            account: &self.account,
            units: &self.units,
        }
    }
}

impl<'a> Fingerprint<'a> for Transaction<'a> {
    type Fingerprint = TransactionFingerprint<'a>;

    fn fingerprint(&'a self) -> Self::Fingerprint {
        TransactionFingerprint {
            date: &self.date,
            payee: &self.payee,
            postings: self.postings.iter().map(Fingerprint::fingerprint).collect(),
        }
    }
}
