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
    file: &'a LedgerFile,
    start_line: usize,
    end_line: usize,
}

impl<'a> Location<'a> {
    pub fn from(file: &'a LedgerFile, start_line: usize, end_line: usize) -> Self {
        Self {
            file,
            start_line,
            end_line,
        }
    }

    pub fn with_context(&self, lines_context: usize) -> LocationSpan<'a> {
        LocationSpan::from([self.clone()].into_iter(), lines_context).unwrap()
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
    lines_context: usize,
}

impl<'a> LocationSpan<'a> {
    pub fn from(
        locations: impl Iterator<Item = Location<'a>>,
        lines_context: usize,
    ) -> Result<Self, LocationError> {
        let mut locations: Vec<_> = locations.collect();

        let set: HashSet<&LedgerFile> = locations.iter().map(|l| l.file).collect();
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
        let first = self.locations.iter().map(|l| l.start_line).min().unwrap();
        let last = self.locations.iter().map(|l| l.end_line).max().unwrap();

        let all_lines: Vec<_> = self
            .locations
            .iter()
            .map(|l| l.start_line..l.end_line)
            .flatten()
            .collect();

        if first == last {
            writeln!(
                f,
                "--> {}:{}",
                self.locations.first().unwrap().file.path.to_string_lossy(),
                first + 1
            )?;
        } else {
            writeln!(
                f,
                "--> {}:{}-{}",
                self.locations.first().unwrap().file.path.to_string_lossy(),
                first + 1,
                last + 1
            )?;
        }

        // Include context when rendering source code.
        let first = max(self.lines_context, first) - self.lines_context;
        let last = last + self.lines_context;

        for (line_number, line) in self
            .locations
            .first()
            .unwrap()
            .file
            .source
            .lines()
            .enumerate()
            .skip(first)
            .take(last - first)
        {
            if all_lines.contains(&line_number) {
                writeln!(f, "{: >4} | {}", line_number + 1, line.bold())?;
            } else {
                writeln!(f, "     | {}", line)?;
            }
        }

        Ok(())
    }
}

pub trait ToLocationSpan<'a> {
    fn to_span(self, tolerance: usize) -> Vec<LocationSpan<'a>>;
}

impl<'a, T> ToLocationSpan<'a> for T
where
    T: Iterator<Item = Location<'a>>,
{
    fn to_span(self, tolerance: usize) -> Vec<LocationSpan<'a>> {
        let mut locations: Vec<_> = self.collect();
        if locations.len() == 0 {
            return Vec::new();
        }

        // Sort by file first, and line number second.
        locations.sort_by(|a, b| {
            let first = a.file.path.cmp(&b.file.path);
            if first == Ordering::Equal {
                a.start_line.cmp(&b.start_line)
            } else {
                first
            }
        });

        let mut spans = Vec::new();
        let mut temp = Vec::new();

        let mut highest = locations.first().unwrap().start_line + tolerance;
        let mut previous_file = locations.first().unwrap().file;
        for location in locations {
            if location.file == previous_file && location.start_line < highest + tolerance {
                highest = location.end_line;
                temp.push(location);
            } else {
                spans.push(LocationSpan::from(temp.iter().cloned(), 1).unwrap());
                temp.truncate(0);
                highest = location.end_line;
                previous_file = location.file;
                temp.push(location);
            }
        }
        spans.push(LocationSpan::from(temp.iter().cloned(), 1).unwrap());

        spans
    }
}
