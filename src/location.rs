use std::{
    cmp::{max, Ordering},
    collections::HashSet,
    fmt::Display,
};

use crate::ledger::LedgerFile;
use colored::Colorize;
use thiserror::Error;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Location<'a> {
    ledger: &'a LedgerFile,
    start_line: u32,
    end_line: u32,
}

impl<'a> PartialOrd for Location<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.ledger.filename().partial_cmp(&other.ledger.filename()) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.start_line.partial_cmp(&other.start_line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.end_line.partial_cmp(&other.end_line)
    }
}

impl<'a> Ord for Location<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ledger.filename().cmp(&other.ledger.filename()) {
            Ordering::Equal => {}
            x => return x,
        }

        match self.start_line.cmp(&other.start_line) {
            Ordering::Equal => {}
            x => return x,
        }

        self.end_line.cmp(&other.end_line)
    }
}

impl<'a> Location<'a> {
    pub fn from(file: &'a LedgerFile, start_line: u32, end_line: u32) -> Self {
        Self {
            ledger: file,
            start_line,
            end_line,
        }
    }

    pub fn with_context(&self, lines_context: u32) -> LocationSpan<'a> {
        LocationSpan::from([self.clone()].into_iter(), lines_context).unwrap()
    }

    pub fn ledger(&self) -> &LedgerFile {
        self.ledger
    }

    pub fn start(&self) -> u32 {
        self.start_line // + self.source.line_offset()
    }

    pub fn end(&self) -> u32 {
        self.end_line // + self.source.line_offset()
    }
}

impl<'a> Display for Location<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.with_context(1))
    }
}

#[derive(Debug, Error)]
pub enum LocationError {
    #[error("cannot span across disparate files")]
    SpanAcrossFiles,
}

pub struct LocationSpan<'a> {
    locations: Vec<Location<'a>>,
    lines_context: u32,
}

impl<'a> LocationSpan<'a> {
    pub fn from(
        locations: impl Iterator<Item = Location<'a>>,
        lines_context: u32,
    ) -> Result<Self, LocationError> {
        let mut locations: Vec<_> = locations.collect();

        let set: HashSet<&LedgerFile> = locations.iter().map(|l| l.ledger).collect();
        if set.len() > 1 {
            Err(LocationError::SpanAcrossFiles)
        } else {
            locations.sort_by(|a, b| a.start_line.cmp(&b.start_line));
            Ok(LocationSpan {
                locations: locations.to_vec(),
                lines_context,
            })
        }
    }
}

impl<'a> Display for LocationSpan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first = self.locations.iter().map(|l| l.start()).min().unwrap();
        let last = self.locations.iter().map(|l| l.end()).max().unwrap();

        let source_ledger = self.locations.first().unwrap().ledger;

        if first == last {
            writeln!(
                f,
                "--> {}:{}",
                source_ledger.filename().to_string_lossy(),
                first + 1 + source_ledger.line_offset()
            )?;
        } else {
            writeln!(
                f,
                "--> {}:{}-{}",
                source_ledger.filename().to_string_lossy(),
                first + 1 + source_ledger.line_offset(),
                last + 1 + source_ledger.line_offset()
            )?;
        }

        // Include context when rendering source code.
        let first = max(self.lines_context, first) - self.lines_context;
        let last = last + self.lines_context;

        for (line_number, line) in source_ledger
            .text_contents
            .lines()
            .enumerate()
            .skip(first as usize)
            .take((last - first) as usize)
        {
            if self
                .locations
                .iter()
                .flat_map(|l| l.start_line..l.end_line)
                .any(|line| line as usize == line_number)
            {
                writeln!(
                    f,
                    "{: >4} | {}",
                    line_number as u32 + 1 + source_ledger.line_offset(),
                    line.bold()
                )?;
            } else {
                writeln!(f, "     | {}", line)?;
            }
        }

        Ok(())
    }
}

pub trait ToLocationSpan<'a> {
    fn to_span(self, tolerance: u32) -> Vec<LocationSpan<'a>>;
}

impl<'a, T> ToLocationSpan<'a> for T
where
    T: Iterator<Item = Location<'a>>,
{
    fn to_span(self, tolerance: u32) -> Vec<LocationSpan<'a>> {
        let mut locations: Vec<_> = self.collect();
        if locations.is_empty() {
            return Vec::new();
        }

        // Sort by file first, and line number second.
        locations.sort_by(|a, b| {
            let first = a.ledger.filename().cmp(&b.ledger.filename());
            if first == Ordering::Equal {
                a.start_line.cmp(&b.start_line)
            } else {
                first
            }
        });

        let mut spans = Vec::new();
        let mut temp = Vec::new();

        let mut highest = locations.first().unwrap().start_line + tolerance;
        let mut previous_file = locations.first().unwrap().ledger;
        for location in locations {
            if location.ledger == previous_file && location.start_line < highest + tolerance {
                highest = location.end_line;
                temp.push(location);
            } else {
                spans.push(LocationSpan::from(temp.iter().cloned(), 1).unwrap());
                temp.truncate(0);
                highest = location.end_line;
                previous_file = location.ledger;
                temp.push(location);
            }
        }
        spans.push(LocationSpan::from(temp.iter().cloned(), 1).unwrap());

        spans
    }
}
