# CSV Cleaner Example (Rust)

**Result: a messy CSV export becomes a clean file plus a report of exactly what was fixed or dropped, in one command (15 tests passing).**

| | Before (by hand) | After (one command) |
|---|---|---|
| Stray spaces | Invisible, break lookups | Trimmed in every cell and header |
| Duplicate rows | Found by eye or by formula | Removed, counted in the report |
| Bad or missing values | Silent mistakes downstream | Row dropped, error counted per column |

A small command-line tool for anyone who receives CSV exports (customers, orders, inventory) and has to clean them before they can be used. It reads a CSV and:

- trims leading and trailing spaces in the header and in every cell;
- checks **required** columns and **types** (`string`, `integer`, `float`, `bool`);
- removes **exact duplicate** rows (the first one is kept);
- writes the clean CSV and prints a report (text on screen, JSON file on request).

All data in the examples is fictional. Written in safe Rust (no `unsafe`), using `clap`, `csv`, `serde` and `toml`.

## Usage

```bash
cargo run --release -- examples/customers.csv --rules examples/rules.toml \
    --output examples/customers_clean.csv --report-json examples/report.json
```

No rules file? Use simple flags instead:

```bash
csv-cleaner input.csv -o clean.csv --required id,email --type id=integer,balance=float
```

| Option | Meaning |
|---|---|
| `<INPUT>` | CSV file; first line is the header |
| `-o, --output` | where to write the cleaned CSV |
| `--rules` | TOML file with one `[[columns]]` block per column (`name`, `type`, `required`) |
| `--required a,b` | columns that must not be empty (when not using `--rules`) |
| `--type a=integer,b=float` | column types (when not using `--rules`) |
| `--report-json` | also write the report as JSON |

Empty optional cells are allowed; a typed column only checks cells that have a value. If a rule names a column that is not in the header, the tool stops with a clear error.

## Example

Input: [`examples/customers.csv`](examples/customers.csv), 8 data rows with stray spaces, a duplicate, a missing e-mail, a text age and a text balance. Rules: [`examples/rules.toml`](examples/rules.toml).

```
Rows read:            8
Rows written:         4
Duplicates removed:   1
Invalid rows removed: 3
Errors by column:
  age: 1
  balance: 1
  email: 1
```

Clean output:

```
id,name,email,age,balance
1,Ana Silva,ana@example.com,34,120.50
2,Ben Carter,ben@example.com,28,80
6,Faye Tanaka,faye@example.com,19,0
7,Gus Moreno,gus@example.com,60,300.10
```

JSON report (`--report-json`):

```json
{
  "rows_read": 8,
  "rows_written": 4,
  "duplicates_removed": 1,
  "invalid_rows_removed": 3,
  "errors_by_column": { "age": 1, "balance": 1, "email": 1 }
}
```

## Tests

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test        # 13 unit tests + 2 integration tests
```

Unit tests cover type checking, rule parsing, trimming, duplicates, required values and malformed rows. The integration test runs the real binary on `examples/customers.csv` and checks the output file and the JSON report. GitHub Actions runs the same three commands on every push.

## Easy to adapt

Typical customizations for a client: extra rules (e-mail format, value ranges, allowed lists), de-duplication by key column instead of the whole row, writing rejected rows to a separate file, Excel (`.xlsx`) input, other delimiters or encodings, or a Python / web wrapper around the same logic.

## License

MIT

## Need something like this?

Built by Gabriel Montemór · Marchenmond. Available on Upwork for small fixes, data cleaning and automations: send a message there and describe your file.
