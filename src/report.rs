//! The summary of what a cleaning run did.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Counts describing one cleaning run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub rows_read: usize,
    pub rows_written: usize,
    pub duplicates_removed: usize,
    pub invalid_rows_removed: usize,
    /// Number of problems found per column (a row can have several).
    pub errors_by_column: BTreeMap<String, usize>,
}

impl Report {
    pub(crate) fn add_error(&mut self, column: &str) {
        *self.errors_by_column.entry(column.to_string()).or_insert(0) += 1;
    }

    /// Human-readable summary.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Rows read:            {}", self.rows_read);
        let _ = writeln!(out, "Rows written:         {}", self.rows_written);
        let _ = writeln!(out, "Duplicates removed:   {}", self.duplicates_removed);
        let _ = writeln!(out, "Invalid rows removed: {}", self.invalid_rows_removed);
        if self.errors_by_column.is_empty() {
            let _ = writeln!(out, "Errors by column:     none");
        } else {
            let _ = writeln!(out, "Errors by column:");
            for (column, count) in &self.errors_by_column {
                let _ = writeln!(out, "  {column}: {count}");
            }
        }
        out
    }

    /// Pretty-printed JSON summary.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report is always serializable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_lists_errors() {
        let mut report = Report {
            rows_read: 3,
            ..Report::default()
        };
        report.add_error("age");
        report.add_error("age");
        let text = report.to_text();
        assert!(text.contains("Rows read:            3"));
        assert!(text.contains("age: 2"));
    }

    #[test]
    fn json_has_expected_fields() {
        let json = Report::default().to_json();
        assert!(json.contains("\"rows_read\": 0"));
        assert!(json.contains("\"errors_by_column\": {}"));
    }
}
