//! Validation rules: which columns are required and what type their values must have.

use serde::Deserialize;
use std::str::FromStr;

/// The kind of value a column may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    /// Any text (always valid).
    #[default]
    String,
    Integer,
    Float,
    Bool,
}

impl ColumnType {
    /// Returns true when `value` (already trimmed, non-empty) fits this type.
    pub fn accepts(self, value: &str) -> bool {
        match self {
            ColumnType::String => true,
            ColumnType::Integer => value.parse::<i64>().is_ok(),
            ColumnType::Float => value.parse::<f64>().map(f64::is_finite).unwrap_or(false),
            ColumnType::Bool => {
                value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false")
            }
        }
    }
}

impl FromStr for ColumnType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "string" => Ok(ColumnType::String),
            "integer" => Ok(ColumnType::Integer),
            "float" => Ok(ColumnType::Float),
            "bool" => Ok(ColumnType::Bool),
            other => Err(format!(
                "unknown type '{other}' (use string, integer, float or bool)"
            )),
        }
    }
}

/// Rule for a single column.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ColumnRule {
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: ColumnType,
    #[serde(default)]
    pub required: bool,
}

/// The full rule set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub columns: Vec<ColumnRule>,
}

impl Rules {
    /// Parses rules from the text of a TOML file.
    pub fn from_toml_str(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// Builds rules from simple flags: `required` column names and `name=type` pairs.
    pub fn from_flags(required: &[String], types: &[String]) -> Result<Self, String> {
        let mut rules = Rules::default();
        for name in required {
            rules.rule_mut(name.trim()).required = true;
        }
        for pair in types {
            let (name, kind) = pair
                .split_once('=')
                .ok_or_else(|| format!("expected name=type, got '{pair}'"))?;
            rules.rule_mut(name.trim()).kind = kind.parse()?;
        }
        Ok(rules)
    }

    fn rule_mut(&mut self, name: &str) -> &mut ColumnRule {
        if let Some(pos) = self.columns.iter().position(|c| c.name == name) {
            return &mut self.columns[pos];
        }
        self.columns.push(ColumnRule {
            name: name.to_string(),
            kind: ColumnType::String,
            required: false,
        });
        self.columns.last_mut().expect("just pushed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_accept_expected_values() {
        assert!(ColumnType::Integer.accepts("-42"));
        assert!(!ColumnType::Integer.accepts("4.2"));
        assert!(ColumnType::Float.accepts("4.25"));
        assert!(!ColumnType::Float.accepts("NaN"));
        assert!(ColumnType::Bool.accepts("TRUE"));
        assert!(!ColumnType::Bool.accepts("yes"));
        assert!(ColumnType::String.accepts("anything"));
    }

    #[test]
    fn parses_toml_rules() {
        let text = r#"
            [[columns]]
            name = "age"
            type = "integer"
            required = true
        "#;
        let rules = Rules::from_toml_str(text).unwrap();
        assert_eq!(rules.columns.len(), 1);
        assert_eq!(rules.columns[0].kind, ColumnType::Integer);
        assert!(rules.columns[0].required);
    }

    #[test]
    fn builds_rules_from_flags() {
        let rules = Rules::from_flags(
            &["email".to_string(), "age".to_string()],
            &["age=integer".to_string()],
        )
        .unwrap();
        assert_eq!(rules.columns.len(), 2);
        let age = rules.columns.iter().find(|c| c.name == "age").unwrap();
        assert!(age.required);
        assert_eq!(age.kind, ColumnType::Integer);
    }

    #[test]
    fn rejects_bad_flags() {
        assert!(Rules::from_flags(&[], &["age".to_string()]).is_err());
        assert!(Rules::from_flags(&[], &["age=number".to_string()]).is_err());
    }
}
