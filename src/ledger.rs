use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use beancount_core::Directive;
use beancount_parser::parse;
use chrono::Local;

use crate::error::Error;

struct LedgerFile {
    path: PathBuf,
    source: String,
}

impl Debug for LedgerFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LedgerFile")
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Debug)]
struct Location<'a> {
    file: &'a LedgerFile,
    line_number: usize,
}

impl<'a> Location<'a> {
    pub fn with_context(&'a self, lines_context: usize) -> LocationContext<'a> {
        LocationContext {
            location: &self,
            lines_context,
        }
    }
}

impl<'a> Display for Location<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.file.source.lines().nth(self.line_number).unwrap();

        writeln!(f, "* {: >6} | {}", self.line_number, line)
    }
}

struct LocationContext<'a> {
    location: &'a Location<'a>,
    lines_context: usize,
}

impl<'a> Display for LocationContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (line_number, line) in self
            .location
            .file
            .source
            .lines()
            .enumerate()
            .skip(max(self.lines_context, self.location.line_number) - self.lines_context)
            .take(self.lines_context * 2 + 1)
        {
            if line_number == self.location.line_number {
                writeln!(f, "* {: >6} | {}", line_number, line)?;
            } else {
                writeln!(f, "  {: >6} | {}", line_number, line)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Sourced<'a, T> {
    directive: T,
    location: Location<'a>,
}

impl<'a, T: Debug> Display for Sourced<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location.with_context(2))
    }
}

/// A Ledger is a complete view of an entire directory structure potentially
/// containing multiple beancount and csv files.
#[derive(Debug)]
pub struct Ledger {
    files: Vec<LedgerFile>,
}

impl Ledger {
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

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut ledgers = Vec::new();
        //let mut imports = Vec::new();

        for file in Self::list_files(path.as_ref())? {
            if let Some(ext) = file.extension() {
                if ext == "beancount" {
                    // Parsing fails if there are no trailing newlines.
                    let source = {
                        let mut source = std::fs::read_to_string(&file)?;
                        source.push('\n');
                        source
                    };

                    ledgers.push(LedgerFile { path: file, source });
                }
            }
        }

        Ok(Ledger { files: ledgers })
    }

    pub fn directives<'a>(&'a self) -> Vec<Sourced<'a, Directive<'a>>> {
        self.files
            .iter()
            .map(LedgerFile::directives)
            .flatten()
            .collect()
    }
}

impl LedgerFile {
    fn directives<'a>(&'a self) -> Vec<Sourced<'a, Directive<'a>>> {
        let ledger = parse(&self.source)
            .map_err(|e| Error::Ledger(self.path.clone(), e))
            .unwrap();

        // We use this hashmap to keep track of each occurrence of a directive.
        // In the case that identical directive occurs multiple times, such as duplicated
        // entries, we'll need to count each occurrence and map them to a single Directive
        // instance.
        let mut occurrences = HashMap::new();
        ledger
            .directives
            .into_iter()
            .map(|directive| {
                let source_text = match &directive {
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
                }
                .unwrap();

                // If this is a duplicate entry, either bump the occurrence or set this as the first one.
                let occurrence = *occurrences
                    .entry(source_text)
                    .and_modify(|entry| *entry += 1)
                    .or_insert(0);

                let source_lines: Vec<&str> = source_text.lines().collect();

                // Find the occurrenceth instance of source_text in source, and collect the line number.
                let line_number = self
                    .source
                    .lines()
                    .enumerate()
                    // This is a bit hacky. Basically we're trying to find the directive substring
                    // in the source string, and retrieving the starting line number.
                    .scan((0, 0), |(index, start), (line_number, line)| {
                        if *index >= source_lines.len() {
                            *index = 0;
                            return Some(Some(*start));
                        } else {
                            if source_lines[*index] == line {
                                *index += 1;
                            } else {
                                *index = 0;
                                *start = line_number + 1;
                            }
                        }

                        Some(None)
                    })
                    .filter_map(Option::from)
                    .nth(occurrence);

                if line_number == None {
                    panic!("Unable to find {:#?}", directive);
                }

                Sourced {
                    directive,
                    location: Location {
                        file: self,
                        line_number: line_number.unwrap(),
                    },
                }
            })
            .collect()
    }
}
