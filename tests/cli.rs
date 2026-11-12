use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn cleans_the_example_file() {
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let cleaned = out_dir.join("customers_clean.csv");
    let report_json = out_dir.join("customers_report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_csv-cleaner"))
        .args(["examples/customers.csv", "--rules", "examples/rules.toml"])
        .arg("--output")
        .arg(&cleaned)
        .arg("--report-json")
        .arg(&report_json)
        .output()
        .expect("binary runs");
    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Rows read:            8"));
    assert!(stdout.contains("Rows written:         4"));

    let csv_text = fs::read_to_string(&cleaned).unwrap();
    assert_eq!(
        csv_text,
        "id,name,email,age,balance\n\
         1,Ana Silva,ana@example.com,34,120.50\n\
         2,Ben Carter,ben@example.com,28,80\n\
         6,Faye Tanaka,faye@example.com,19,0\n\
         7,Gus Moreno,gus@example.com,60,300.10\n"
    );

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_json).unwrap()).unwrap();
    assert_eq!(json["duplicates_removed"], 1);
    assert_eq!(json["invalid_rows_removed"], 3);
    assert_eq!(json["errors_by_column"]["email"], 1);
    assert_eq!(json["errors_by_column"]["age"], 1);
    assert_eq!(json["errors_by_column"]["balance"], 1);
}

#[test]
fn fails_on_unknown_column() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("never.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_csv-cleaner"))
        .args(["examples/customers.csv", "--required", "nope"])
        .arg("--output")
        .arg(out)
        .output()
        .expect("binary runs");
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr).unwrap().contains("nope"));
}
