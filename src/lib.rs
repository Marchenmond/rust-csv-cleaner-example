//! Clean a CSV stream: trim whitespace, validate columns, drop duplicates.

mod report;
mod rules;

pub use report::Report;
pub use rules::{ColumnRule, ColumnType, Rules};

use std::collections::HashSet;
use std::fmt;
use std::io::{Read, Write};

/// Label used in the report for rows whose number of fields does not match the header.
pub const ROW_SHAPE: &str = "(row shape)";

/// Things that stop a run entirely.
#[derive(Debug)]
pub enum CleanError {
    /// The CSV could not be read or written.
    Csv(csv::Error),
    /// The rules mention columns that are not in the header.
    MissingColumns(Vec<String>),
}

impl fmt::Display for CleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanError::Csv(e) => write!(f, "CSV error: {e}"),
            CleanError::MissingColumns(cols) => {
                write!(f, "columns not found in header: {}", cols.join(", "))
            }
        }
    }
}

impl std::error::Error for CleanError {}

impl From<csv::Error> for CleanError {
    fn from(e: csv::Error) -> Self {
        CleanError::Csv(e)
    }
}

/// Reads CSV from `input`, writes the cleaned CSV to `output`, and returns a report.
///
/// Steps per row: trim every field, validate against `rules`, then skip exact
/// duplicates (the first occurrence wins). Rows that fail validation are removed.
pub fn clean<R: Read, W: Write>(input: R, output: W, rules: &Rules) -> Result<Report, CleanError> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(input);
    let mut writer = csv::Writer::from_writer(output);

    let headers: Vec<String> = reader
        .headers()?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let missing: Vec<String> = rules
        .columns
        .iter()
        .filter(|rule| !headers.contains(&rule.name))
        .map(|rule| rule.name.clone())
        .collect();
    if !missing.is_empty() {
        return Err(CleanError::MissingColumns(missing));
    }

    // Position of each rule's column, resolved once.
    let checks: Vec<(usize, &ColumnRule)> = rules
        .columns
        .iter()
        .filter_map(|rule| {
            headers
                .iter()
                .position(|h| *h == rule.name)
                .map(|i| (i, rule))
        })
        .collect();

    writer.write_record(&headers)?;

    let mut report = Report::default();
    let mut seen: HashSet<Vec<String>> = HashSet::new();

    for record in reader.records() {
        let record = record?;
        report.rows_read += 1;
        let row: Vec<String> = record.iter().map(|f| f.trim().to_string()).collect();

        if row.len() != headers.len() {
            report.add_error(ROW_SHAPE);
            report.invalid_rows_removed += 1;
            continue;
        }

        let mut valid = true;
        for (index, rule) in &checks {
            let value = &row[*index];
            if value.is_empty() {
                if rule.required {
                    report.add_error(&rule.name);
                    valid = false;
                }
            } else if !rule.kind.accepts(value) {
                report.add_error(&rule.name);
                valid = false;
            }
        }
        if !valid {
            report.invalid_rows_removed += 1;
            continue;
        }

        if seen.contains(&row) {
            report.duplicates_removed += 1;
            continue;
        }
        writer.write_record(&row)?;
        seen.insert(row);
        report.rows_written += 1;
    }

    writer.flush().map_err(csv::Error::from)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str, rules: &Rules) -> (Result<Report, CleanError>, String) {
        let mut out = Vec::new();
        let result = clean(input.as_bytes(), &mut out, rules);
        (result, String::from_utf8(out).unwrap())
    }

    fn rules(required: &[&str], types: &[&str]) -> Rules {
        let required: Vec<String> = required.iter().map(|s| s.to_string()).collect();
        let types: Vec<String> = types.iter().map(|s| s.to_string()).collect();
        Rules::from_flags(&required, &types).unwrap()
    }

    #[test]
    fn trims_headers_and_fields() {
        let (result, out) = run(" a , b \n x , y \n", &Rules::default());
        assert_eq!(out, "a,b\nx,y\n");
        assert_eq!(result.unwrap().rows_written, 1);
    }

    #[test]
    fn removes_duplicates_after_trimming() {
        let (result, out) = run("a,b\n1,x\n1, x \n2,y\n", &Rules::default());
        let report = result.unwrap();
        assert_eq!(out, "a,b\n1,x\n2,y\n");
        assert_eq!(report.duplicates_removed, 1);
        assert_eq!(report.rows_read, 3);
    }

    #[test]
    fn drops_rows_missing_required_values() {
        let (result, out) = run("a,b\n1,\n2,y\n", &rules(&["b"], &[]));
        let report = result.unwrap();
        assert_eq!(out, "a,b\n2,y\n");
        assert_eq!(report.invalid_rows_removed, 1);
        assert_eq!(report.errors_by_column["b"], 1);
    }

    #[test]
    fn drops_rows_with_wrong_types() {
        let (result, out) = run("n\n5\nabc\n", &rules(&[], &["n=integer"]));
        let report = result.unwrap();
        assert_eq!(out, "n\n5\n");
        assert_eq!(report.errors_by_column["n"], 1);
    }

    #[test]
    fn optional_empty_values_are_allowed() {
        let (result, out) = run("a,n\nx,\n", &rules(&[], &["n=integer"]));
        assert_eq!(result.unwrap().rows_written, 1);
        assert_eq!(out, "a,n\nx,\n");
    }

    #[test]
    fn counts_rows_with_wrong_field_count() {
        let (result, out) = run("a,b\n1,2,3\n4,5\n", &Rules::default());
        let report = result.unwrap();
        assert_eq!(out, "a,b\n4,5\n");
        assert_eq!(report.errors_by_column[ROW_SHAPE], 1);
    }

    #[test]
    fn missing_rule_column_is_an_error() {
        let (result, _) = run("a\n1\n", &rules(&["zzz"], &[]));
        match result {
            Err(CleanError::MissingColumns(cols)) => assert_eq!(cols, vec!["zzz"]),
            other => panic!("unexpected: {other:?}"),
        }
    }
}
