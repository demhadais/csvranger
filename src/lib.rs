#[cfg(feature = "legacy")]
pub use legacy::{
    LEGACY_CELLRANGERMULTI_CSV_VALUE_REGEX, parse_legacy_csv_value_as_f64,
    parse_legacy_csv_value_as_i32,
};
use std::str::FromStr;

#[cfg(feature = "legacy")]
mod legacy;

/// A CSV value found in 10x the output `metrics_summary.csv` produced by 10x Genomics *ranger pipelines.
///
/// One such file contains many such values with mixed types, so if parsing a whole CSV at once, it's useful to have one type that can fit any of the values.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum TenxCsvValue {
    F64(f64),
    I32(i32),
    String(String),
}

impl TenxCsvValue {
    /// Parse a value from a CSV produced by a modern *ranger pipeline.
    ///
    /// Constructing a value through this method bypasses any regular expression checking and assumes the string provided is numeric (that is, all characters are digits or '.') or is meant to be a string. If you need to parse strings with extraneous characters produced by older *ranger pipelines (like "310,209 (95.3%)", "310,209", or "50.0%"), use [`TenxCsvValue::from_legacy_csv_value`] instead.
    pub fn from_csv_value(val: &str) -> Self {
        i32::from_str(val)
            .ok()
            .map(Self::I32)
            .or_else(|| f64::from_str(val).ok().map(Self::F64))
            .unwrap_or_else(|| Self::String(val.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn qc_library_metrics() {}
}
