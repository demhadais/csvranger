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
    /// If the CSV was generated using any of the following, use [`TenxCsvValue::from_legacy_csv_value`] instead to correctly extract numerical values (you will need to activate the `legacy` feature):
    /// - cellranger count < 10
    /// - cellranger multi < 10
    ///
    /// Otherwise, use this constructor.
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
    use crate::TenxCsvValue;

    fn read_multi_row_csv(raw_csv: &[u8]) -> Vec<TenxCsvValue> {
        let mut parsed_data = Vec::with_capacity(19);

        let mut reader = csv::Reader::from_reader(raw_csv);
        for line in reader.records() {
            let line = line.unwrap();
            let raw_val = line.get(5).unwrap();
            parsed_data.push(TenxCsvValue::from_csv_value(raw_val));
        }

        parsed_data
    }

    #[test]
    fn qc_library_metrics() {
        let data = include_bytes!(
            "../test-data/cellranger_multi.10.0/SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_library_metrics.csv"
        );
        let parsed_data = read_multi_row_csv(&data[..]);
    }
}
