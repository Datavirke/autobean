mod error;
mod ledger;

fn main() {
    let ledger = Ledger::from_path("data").unwrap();

    for i in ledger.directives().iter().take(5) {
        println!("{}", i);
    }
}
/*
mod ingest;

use std::{
    borrow::Cow,
    collections::{HashMap},
    path::{Path, PathBuf},
};

use beancount_core::{Account, Date, Directive, IncompleteAmount, Ledger, Open, Transaction};
use beancount_parser::{error::ParseError, parse};
use beancount_render::render;

use crate::ingest::ALTransaction;
use thiserror::Error;

fn list_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    fn visit_files_internal(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_files_internal(&path, files)?;
                } else {
                    files.push(entry.path())
                }
            }
        } else {
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit_files_internal(dir, &mut files).map(|_| files)
}

#[derive(Debug, Error)]
enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("loading ledger {0}, {1}")]
    Ledger(PathBuf, ParseError),
    #[error("import {0}")]
    Import(#[from] csv::Error),
}

#[derive(Debug)]
struct LedgerFile {
    path: PathBuf,
    source: String,
}

struct Location<'a> {
    ledger: &'a LedgerFile,
    line_number: usize,
}

struct SourcedDirective<'a> {
    directive: Directive<'a>,
    location: Location<'a>,
}

impl LedgerFile {
    fn reload<'a>(&'a mut self) -> std::io::Result<()> {
        self.source = {
            let mut source = std::fs::read_to_string(&self.path)?;
            source.push('\n');
            source
        };

        Ok(())
    }

    fn read<'a>(&'a self) -> std::io::Result<Ledger<'a>> {
        let ledger = parse(&self.source)
            .map_err(|e| Error::Ledger(self.path.clone(), e))
            .unwrap();

        Ok(ledger)
    }

    fn read_directives<'a>(&'a self) -> std::io::Result<Vec<Directive<'a>>> {
        let ledger = parse(&self.source)
            .map_err(|e| Error::Ledger(self.path.clone(), e))
            .unwrap();

        Ok(ledger.directives)
    }
    /*
    fn source<'a>(&'a self, dir: Directive<'a>) -> Vec<Sourced<'a, Directive<'a>>> {
        let text = match dir {
            Directive::Open(inner) => inner.source,
            Directive::Close(inner) => inner.source,
            Directive::Balance(inner) => inner.source,
            Directive::Option(inner) => inner.source,
            Directive::Commodity(inner) => inner.source,
            Directive::Custom(inner) => inner.source,
            Directive::Document(inner) => inner.source,
            Directive::Event(inner) => inner.source,
            Directive::Include(inner) => inner.source,
            Directive::Note(inner) => inner.source,
            Directive::Pad(inner) => inner.source,
            Directive::Plugin(inner) => inner.source,
            Directive::Price(inner) => inner.source,
            Directive::Query(inner) => inner.source,
            Directive::Transaction(inner) => inner.source,
            Directive::Unsupported => None,
        }?;

        let occurrences = self.source.lines().enumerate().filter_map(|(number, line)| if line == text { Some(Sourced {
            directive: dir,
            location: Location {
                ledger: self,
                line: number
            }
        }) } else { None }).collect();
    }
     */
}

#[derive(Debug)]
struct TransactionFile {
    path: PathBuf,
    data: Vec<ALTransaction>,
}

#[derive(Debug)]
struct Everything {
    ledgers: Vec<LedgerFile>,
    imports: Vec<TransactionFile>,
}

#[derive(Debug)]
struct Sourced<T> {
    directive: T,
}

impl Everything {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Everything, Error> {
        let mut ledgers = Vec::new();
        let mut imports = Vec::new();

        for file in list_files(path.as_ref())? {
            if let Some(ext) = file.extension() {
                if ext == "beancount" {
                    // Parsing fails if there are no trailing newlines.
                    let source = {
                        let mut source = std::fs::read_to_string(&file)?;
                        source.push('\n');
                        source
                    };

                    ledgers.push(LedgerFile { path: file, source });
                } else if ext == "csv" {
                    let reader = std::fs::File::open(&file).unwrap();
                    let mut deser = csv::Reader::from_reader(&reader);

                    let transactions: Result<Vec<ALTransaction>, _> = deser.deserialize().collect();
                    imports.push(TransactionFile {
                        path: file,
                        data: transactions?,
                    })
                }
            }
        }

        Ok(Everything { ledgers, imports })
    }

    fn directives<'a>(&'a self) -> Vec<SourcedDirective<'a>> {
        let mut directives = Vec::new();
        for file in self.ledgers.iter() {
            let ledger = file.read().unwrap();
            for directive in ledger.directives {
                directives.push(SourcedDirective {
                    directive,
                    location: Location {
                        ledger: file,
                        line_number: 0
                    }
                })
            }
        }

        directives
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PostingFingerprint<'a> {
    account: &'a Account<'a>,
    units: &'a IncompleteAmount<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TransactionFingerprint<'a> {
    date: &'a Date<'a>,
    payee: &'a Option<Cow<'a, str>>,
    postings: Vec<PostingFingerprint<'a>>,
}

fn find_duplicates(everything: &Everything) {
    let mut set = HashMap::new();

    let mut directives = everything.directives();
    for dir in &mut directives {
        if let Directive::Transaction(txn) = &mut dir.directive {
            set.entry(TransactionFingerprint {
                date: &txn.date,
                payee: &txn.payee,
                postings: txn
                    .postings
                    .iter()
                    .map(|p| PostingFingerprint {
                        account: &p.account,
                        units: &p.units,
                    })
                    .collect(),
            })
            .and_modify(|sources: &mut Vec<&Path>| sources.push(&dir.location.ledger.path))
            .or_insert(vec![&dir.location.ledger.path]);
        }
    }

    for (txn, sources) in set.iter().filter(|(_, sources)| sources.len() > 1) {
        println!(
            "{:?} found in multiple locations:",
            txn.payee.as_deref().unwrap_or_default()
        );
        for source in sources {
            println!("    {}", source.to_str().unwrap())
        }
    }
}

fn main() {
    let everything = Everything::from_path(".").unwrap();

    find_duplicates(&everything);

    //let file = std::fs::File::open("lønkonto-2020.csv").unwrap();
    //let mut deser = csv::Reader::from_reader(file);

    //let transactions: Vec<ALTransaction> = deser.deserialize().map(Result::unwrap).collect();

    //println!("{:?}", transactions);

    //match run() {
    //        Err(e) => println!("Error: {}", e),
    //      _ => {}
    //}
}
 */

use ledger::Ledger;
