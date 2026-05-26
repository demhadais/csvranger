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
    /// If the CSV-file was generated using any of the following, use [`TenxCsvValue::from_legacy_csv_value`] instead to correctly extract numerical values (you will need to activate the `legacy` feature):
    /// - `cellranger count < 10`
    /// - `cellranger multi < 10`
    ///
    /// If the CSV-file was generated using any of the following:
    /// - `cellranger count >= 10`
    /// - `cellranger multi >= 10`
    /// - `cellranger-atac count >= 2`
    /// - `spaceranger count >= 4`
    ///
    /// then this is the correct method. Note that [`TenxCsvValue::from_legacy_csv_value`] will still work for modern pipelines, but because it uses a regular expression for parsing, it will be less performant. Note also that other versions of `spaceranger` and `cellranger-atac` may also produce outputs compatible with this method, but they are currently untested.
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
    fn cellranger_multi_10_qc_library_metrics() {
        let raw_data = include_bytes!(
            "../test-data/cellranger_multi.10.0/SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_library_metrics.csv"
        );

        let parsed_data = read_multi_row_csv(&raw_data[..]);
        let expected_data = vec![
            TenxCsvValue::I32(321950603),
            TenxCsvValue::F64(0.9754667347931396),
            TenxCsvValue::F64(0.9838510115040644),
            TenxCsvValue::F64(0.9810434401949358),
            TenxCsvValue::I32(16410),
            TenxCsvValue::F64(0.18127985925840928),
            TenxCsvValue::F64(0.6847850295990157),
            TenxCsvValue::F64(0.39215314033749454),
            TenxCsvValue::F64(0.9176445866138042),
            TenxCsvValue::F64(0.07506513662283776),
            TenxCsvValue::F64(0.4504263096534719),
            TenxCsvValue::F64(0.657908592269355),
            TenxCsvValue::F64(0.945846350845319),
            TenxCsvValue::I32(19619),
            TenxCsvValue::I32(321950603),
            TenxCsvValue::I32(321950603),
            TenxCsvValue::F64(0.4625052256865317),
            TenxCsvValue::F64(0.9984212981890268),
            TenxCsvValue::F64(0.8701900924844672),
        ];

        assert_eq!(parsed_data, expected_data);
    }
}
