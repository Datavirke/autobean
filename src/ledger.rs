use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    path::{Path, PathBuf},
};

use beancount_core::Directive;
use beancount_parser::parse;
use colored::Colorize;

use crate::{error::Error, location::Location};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LedgerSource {
    File(PathBuf),
    #[cfg(test)]
    Code {
        filename: &'static str,
        line_offset: u32,
    },
}

#[macro_export]
macro_rules! inline_ledger {
    ( $source:literal ) => {
        crate::ledger::Ledger {
            files: vec![crate::ledger::LedgerFile {
                source: crate::ledger::LedgerSource::Code {
                    filename: file!(),
                    line_offset: line!() + 1,
                },
                text_contents: unindent::unindent($source),
            }],
        }
    };
}

impl LedgerSource {
    pub fn filename(&self) -> PathBuf {
        match self {
            LedgerSource::File(path) => path.clone(),
            #[cfg(test)]
            LedgerSource::Code { filename, .. } => PathBuf::from(filename),
        }
    }

    pub fn line_offset(&self) -> u32 {
        match self {
            LedgerSource::File(_) => 0,
            #[cfg(test)]
            LedgerSource::Code { line_offset, .. } => *line_offset,
        }
    }
}

impl From<PathBuf> for LedgerSource {
    fn from(path: PathBuf) -> Self {
        LedgerSource::File(path)
    }
}

#[derive(Eq)]
pub struct LedgerFile {
    pub source: LedgerSource,
    pub text_contents: String,
}

impl Hash for LedgerFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
    }
}

impl PartialEq for LedgerFile {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Debug for LedgerFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Omit the 'source' string, since it's usually very large.
        f.debug_struct("LedgerFile")
            .field("path", &self.source)
            .finish()
    }
}

#[derive(Clone)]
pub struct Sourced<'a, T> {
    pub inner: T,
    pub location: Location<'a>,
}

impl<'a, T> Eq for Sourced<'a, T> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<'a, T> PartialEq for Sourced<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl<'a, T> Hash for Sourced<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Only hash location, since we assume that must be unique
        self.location.hash(state);
    }
}

impl<'a, T> Debug for Sourced<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sourced")
            .field("location", &self.location)
            .finish()
    }
}

impl<'a, T> Display for Sourced<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location.with_context(1))
    }
}

/// A Ledger is a complete view of an entire directory structure potentially
/// containing multiple beancount and csv files.
#[derive(Debug)]
pub struct Ledger {
    pub files: Vec<LedgerFile>,
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

        for file in Self::list_files(path.as_ref())? {
            if let Some(ext) = file.extension() {
                if ext == "beancount" {
                    // Parsing fails if there are no trailing newlines.
                    let source = {
                        let mut source = std::fs::read_to_string(&file)?;
                        source.push('\n');
                        source
                    };

                    ledgers.push(LedgerFile {
                        source: file.into(),
                        text_contents: source,
                    });
                }
            }
        }

        Ok(Ledger { files: ledgers })
    }

    pub fn directives(&self) -> Vec<Sourced<'_, Directive<'_>>> {
        self.files.iter().flat_map(LedgerFile::directives).collect()
    }
}

impl LedgerFile {
    pub fn filename(&self) -> PathBuf {
        self.source.filename()
    }

    pub fn line_offset(&self) -> u32 {
        self.source.line_offset()
    }

    fn directives(&self) -> Vec<Sourced<'_, Directive<'_>>> {
        let ledger = parse(&self.text_contents)
            .map_err(|e| Error::Ledger(self.source.filename(), e))
            .unwrap();

        // We use this hashmap to keep track of each occurrence of a directive.
        // In the case that an identical directive occurs multiple times, such as duplicated
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
                .expect("Directive did not have a source associated with it");

                // If this is a duplicate entry, either bump the occurrence or set this as the first one.
                let occurrence = *occurrences
                    .entry(source_text)
                    .and_modify(|entry| *entry += 1)
                    .or_insert(0);

                let source_lines: Vec<&str> = source_text.lines().collect();

                let matches: Vec<_> = self
                    .text_contents
                    .lines()
                    .enumerate()
                    // This is a bit hacky. Basically we're trying to find the directive substring
                    // in the source string, and retrieving the start and end line number.
                    .scan((0u32, 0u32), |(index, start), (line_number, line)| {
                        if source_lines[*index as usize] == line {
                            *index += 1;
                        } else {
                            *index = 0;
                            *start = (line_number + 1) as u32;
                        }

                        if *index as usize == source_lines.len() {
                            let lines = (*start, *start + *index);
                            *start += *index + 1;
                            *index = 0;
                            Some(Some(lines))
                        } else {
                            Some(None)
                        }
                    })
                    .filter_map(Option::from)
                    .collect();

                // Find the occurrenceth instance of source_text in source, and collect the line number.
                if let Some((start, end)) = matches.get(occurrence) {
                    Sourced {
                        inner: directive,
                        location: Location::from(self, *start, *end),
                    }
                } else {
                    panic!(
                        "Unable to find\n{}\n in source file {}",
                        source_text.red(),
                        self.filename().to_string_lossy()
                    );
                }
            })
            .collect()
    }
}
