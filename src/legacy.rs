use std::sync::LazyLock;

use regex::Regex;

/// A regular expreession that extracts the meaningful data from CSV values found in the `metrics_summary.csv` files outputted by 10x Genomics *ranger pipelines.
///
/// From a string like "310,209 (95.3%)", this regex extracts "310,209", which is only useful for `cellranger multi <= 10.0`. Because the part in the parethenteses is optional, it also matches strings like "310,209" and "60.10%".
///
/// Note: from `cellranger >= 10.0`, 10x Genomics started formatting numerical values in CSV files as plain numbers, so this regex is only required for parsing older versions.
#[cfg(feature = "legacy")]
pub const LEGACY_CELLRANGERMULTI_CSV_VALUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"([\d,%]+)( \(.*\))?"#).expect("regular expression should be valid")
});

impl super::TenxCsvValue {
    pub fn from_legacy_csv_value(val: &str) -> Self {
        let Some(parsed_val) = parse_legacy_csv_value_as_f64(val) else {
            return Self::String(val.to_owned());
        };

        f64_to_i32(parsed_val)
            .map(Self::I32)
            .unwrap_or(Self::F64(parsed_val))
    }
}

pub fn parse_legacy_csv_value_as_f64(val: &str) -> Option<f64> {
    // Optimistically try to parse the value as a number
    if let Ok(parsed_value) = val.parse() {
        return Some(parsed_value);
    }

    let extracted_str = extract_numeric_part(val)?;

    // We know this string contains commas and/or percent symbols, so remove them before trying to parse as a number again
    let numeric_str = extracted_str.replace([',', '%'], "");

    // If we couldn't parse the transformed string as a number, then it's hopeless
    let parsed_value = numeric_str.parse().ok()?;

    // Now we have a float, but if it was a percentage, we need to divide by 100
    let parsed_value = if extracted_str.contains('%') {
        parsed_value / 100.0
    } else {
        parsed_value
    };

    Some(parsed_value)
}

pub fn parse_legacy_csv_value_as_i32(val: &str) -> Option<i32> {
    parse_legacy_csv_value_as_f64(val).map(f64_to_i32).flatten()
}

fn f64_to_i32(f: f64) -> Option<i32> {
    (f.trunc() == f).then(|| f as i32)
}

fn extract_numeric_part(s: &str) -> Option<&str> {
    LEGACY_CELLRANGERMULTI_CSV_VALUE_REGEX
        .captures(s)
        .map(|c| c.get(1))
        .flatten()
        .map(|m| m.as_str())
}
