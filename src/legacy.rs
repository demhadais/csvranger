use std::sync::LazyLock;

use regex::Regex;

/// A regular expreession that extracts the meaningful data from CSV values found in the `metrics_summary.csv` files outputted by 10x Genomics *ranger pipelines.
///
/// From a string like "310,209 (95.3%)", this regex extracts "310,209", which is only useful for `cellranger multi <= 10.0`. Because the part in the parethenteses is optional, it also matches strings like "310,209" and "60.10%".
///
/// Note: from `cellranger >= 10.0`, 10x Genomics started formatting numerical values in CSV files as plain numbers, so this regex is only required for parsing older versions.
#[cfg(feature = "legacy")]
static LEGACY_CELLRANGERMULTI_CSV_VALUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^([\d,%\.]+)( \(.*\))?$"#).expect("regular expression should be valid")
});

impl super::TenxCsvValue {
    /// Parse a value from a CSV produced by a legacy *ranger pipeline.
    ///
    /// If the CSV-file was generated using any of the following:
    /// - `cellranger count < 10`
    /// - `cellranger multi < 10`
    ///
    /// use this method.
    ///
    /// Otherwise, use [`crate::TenxCsvValue::from_csv_value`].
    pub fn from_legacy_csv_value(val: &str) -> Self {
        let Some(parsed_val) = parse_legacy_csv_value_as_f64(val) else {
            return Self::String(val.to_owned());
        };

        f64_to_i64(parsed_val)
            .map(Self::I64)
            .unwrap_or(Self::F64(parsed_val))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for super::TenxCsvValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(Self::from_legacy_csv_value(&s))
    }
}

/// Parse a value in a CSV outputted by a legacy 10x Genomics pipeline as an `f64`.
///
/// If you only need a field that you know contains a float, you can bypass [`TenxCsvValue`](crate::TenxCsvValue) and just use this function to get an `f64` directly. Note that integer fields will also parse as `f64`, so you can use this function for any fields which you know contain a numeric value.
///
/// # Example
/// ```
/// use csvranger::parse_legacy_csv_value_as_f64;
///
/// assert_eq!(parse_legacy_csv_value_as_f64("92.6%").unwrap(), 92.6 / 100.0);
/// ```
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

/// Parse a value in a CSV outputted by a legacy 10x Genomics pipeline as an `i64`.
///
/// If you only need a field that you know contains an integer, you can bypass [`crate::TenxCsvValue`] and just use this function to get an `i64` directly.
///
/// # Example
/// ```
/// use csvranger::parse_legacy_csv_value_as_i64;
///
/// assert_eq!(parse_legacy_csv_value_as_i64("2,448,314,815").unwrap(), 2_448_314_815);
/// assert_eq!(parse_legacy_csv_value_as_i64("312,195 (100.0%)").unwrap(), 312_195);
/// ```
pub fn parse_legacy_csv_value_as_i64(val: &str) -> Option<i64> {
    parse_legacy_csv_value_as_f64(val).map(f64_to_i64).flatten()
}

fn f64_to_i64(f: f64) -> Option<i64> {
    (f.trunc() == f).then(|| f as i64)
}

fn extract_numeric_part(s: &str) -> Option<&str> {
    LEGACY_CELLRANGERMULTI_CSV_VALUE_REGEX
        .captures(s)
        .map(|c| c.get(1))
        .flatten()
        .map(|m| m.as_str())
}

#[cfg(test)]
mod tests {
    use crate::TenxCsvValue;
    use crate::legacy::extract_numeric_part;

    fn read_legacy_singlerow_csv(raw_csv: &[u8]) -> Vec<TenxCsvValue> {
        crate::tests::read_singlerow_csv(raw_csv, TenxCsvValue::from_legacy_csv_value)
    }

    fn read_legacy_multirow_csv(raw_csv: &[u8]) -> Vec<TenxCsvValue> {
        crate::tests::read_multirow_csv(raw_csv, TenxCsvValue::from_legacy_csv_value)
    }

    #[test]
    fn cellranger_count_6_metrics_summary() {
        let raw_data = include_bytes!(
            "../test-data/cellranger_count.6.0/Breast_Cancer_3p_metrics_summary.csv"
        );

        let parsed_data = read_legacy_singlerow_csv(&raw_data[..]);
        let expected_data = vec![
            TenxCsvValue::I64(5680),
            TenxCsvValue::I64(54504),
            TenxCsvValue::I64(2610),
            TenxCsvValue::I64(309585432),
            TenxCsvValue::F64(98.2 / 100.0),
            TenxCsvValue::F64(33.3 / 100.0),
            TenxCsvValue::F64(95.7 / 100.0),
            TenxCsvValue::F64(93.5 / 100.0),
            TenxCsvValue::F64(94.2 / 100.0),
            TenxCsvValue::F64(96.8 / 100.0),
            TenxCsvValue::F64(94.0 / 100.0),
            TenxCsvValue::F64(8.7 / 100.0),
            TenxCsvValue::F64(21.9 / 100.0),
            TenxCsvValue::F64(63.4 / 100.0),
            TenxCsvValue::F64(60.8 / 100.0),
            TenxCsvValue::F64(1.4 / 100.0),
            TenxCsvValue::F64(93.7 / 100.0),
            TenxCsvValue::I64(26156),
            TenxCsvValue::I64(11498),
        ];

        assert_eq!(parsed_data, expected_data);
    }

    #[test]
    fn cellranger_multi_8_metrics_summary() {
        let raw_data = include_bytes!(
            "../test-data/cellranger_multi.8.0/10k_Mouse_Brain_CNIK_3p_gemx_10k_Mouse_Brain_CNIK_3p_gemx_metrics_summary.csv"
        );

        let parsed_data = read_legacy_multirow_csv(&raw_data[..]);
        let expected_data = vec![
            TenxCsvValue::I64(11357),
            TenxCsvValue::F64(93.55 / 100.0),
            TenxCsvValue::I64(48067),
            TenxCsvValue::I64(13378),
            TenxCsvValue::I64(3502),
            TenxCsvValue::I64(28936),
            TenxCsvValue::I64(619515724),
            TenxCsvValue::I64(0),
            TenxCsvValue::F64(91.9 / 100.0),
            TenxCsvValue::F64(94.8 / 100.0),
            TenxCsvValue::F64(95.3 / 100.0),
            TenxCsvValue::F64(12.48 / 100.0),
            TenxCsvValue::F64(93.55 / 100.0),
            TenxCsvValue::F64(27.48 / 100.0),
            TenxCsvValue::F64(91.80 / 100.0),
            TenxCsvValue::F64(6.01 / 100.0),
            TenxCsvValue::F64(58.31 / 100.0),
            TenxCsvValue::F64(72.87 / 100.0),
            TenxCsvValue::I64(11357),
            TenxCsvValue::F64(94.36 / 100.0),
            TenxCsvValue::I64(54549),
            TenxCsvValue::I64(619515724),
            TenxCsvValue::I64(619515724),
            TenxCsvValue::F64(47.13 / 100.0),
            TenxCsvValue::F64(99.94 / 100.0),
            TenxCsvValue::F64(95.74 / 100.0),
        ];

        assert_eq!(parsed_data, expected_data);
    }

    #[test]
    fn cellranger_multi_9_metrics_summary() {
        let raw_data = include_bytes!(
            "../test-data/cellranger_multi.9.0/320k_K562_Flex_CRISPR_Ultima_320k_K562_Flex_CRISPR_Ultima_metrics_summary.csv"
        );

        let parsed_data = read_legacy_multirow_csv(&raw_data[..]);
        let expected_data = vec![
            TenxCsvValue::I64(312195),
            TenxCsvValue::F64(92.6 / 100.0),
            TenxCsvValue::F64(31.9 / 100.0),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(90.7 / 100.0),
            TenxCsvValue::I64(7726),
            TenxCsvValue::I64(621),
            TenxCsvValue::I64(2448314815),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(312195),
            TenxCsvValue::I64(312195),
            TenxCsvValue::F64(95.0 / 100.0),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(2),
            TenxCsvValue::I64(8218),
            TenxCsvValue::I64(5731),
            TenxCsvValue::I64(3237),
            TenxCsvValue::I64(2565602560),
            TenxCsvValue::F64(95.9 / 100.0),
            TenxCsvValue::F64(97.8 / 100.0),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(2.2 / 100.0),
            TenxCsvValue::I64(16886),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(312195),
            TenxCsvValue::I64(2701778863),
            TenxCsvValue::I64(0),
            TenxCsvValue::F64(93.0 / 100.0),
            TenxCsvValue::F64(94.9 / 100.0),
            TenxCsvValue::F64(96.9 / 100.0),
            TenxCsvValue::F64(91.7 / 100.0),
            TenxCsvValue::F64(88.9 / 100.0),
            TenxCsvValue::I64(312195),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(89.3 / 100.0),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(90.7 / 100.0),
            TenxCsvValue::I64(8654),
            TenxCsvValue::I64(2701778863),
            TenxCsvValue::F64(86.8 / 100.0),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(18579),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21951665),
            TenxCsvValue::I64(19452),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(22141744),
            TenxCsvValue::I64(19309),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(22204657),
            TenxCsvValue::I64(19066),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(20836241),
            TenxCsvValue::I64(19262),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21292446),
            TenxCsvValue::I64(19279),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21003609),
            TenxCsvValue::I64(19157),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(22919935),
            TenxCsvValue::I64(19955),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21338051),
            TenxCsvValue::I64(19999),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21805346),
            TenxCsvValue::I64(20074),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(19879218),
            TenxCsvValue::I64(19893),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21805282),
            TenxCsvValue::I64(20070),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(23094332),
            TenxCsvValue::I64(19516),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(22458721),
            TenxCsvValue::I64(19286),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(21547490),
            TenxCsvValue::I64(19446),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(22473279),
            TenxCsvValue::I64(19852),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(23914539),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(2),
            TenxCsvValue::I64(2743861986),
            TenxCsvValue::I64(59719110),
            TenxCsvValue::F64(92.7 / 100.0),
            TenxCsvValue::F64(90.3 / 100.0),
            TenxCsvValue::F64(96.8 / 100.0),
            TenxCsvValue::F64(92.0 / 100.0),
            TenxCsvValue::F64(90.6 / 100.0),
            TenxCsvValue::I64(312195),
            TenxCsvValue::F64(95.0 / 100.0),
            TenxCsvValue::F64(97.1 / 100.0),
            TenxCsvValue::I64(8789),
            TenxCsvValue::I64(2743861986),
            TenxCsvValue::I64(2743861986),
            TenxCsvValue::F64(95.9 / 100.0),
            TenxCsvValue::F64(97.8 / 100.0),
            TenxCsvValue::I64(0),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(2.2 / 100.0),
            TenxCsvValue::F64(21.1 / 100.0),
            TenxCsvValue::I64(1),
            TenxCsvValue::I64(1),
            TenxCsvValue::F64(98.5 / 100.0),
            TenxCsvValue::F64(98.5 / 100.0),
            TenxCsvValue::I64(18579),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(118779761),
            TenxCsvValue::I64(19452),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(119729069),
            TenxCsvValue::I64(19309),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(125634537),
            TenxCsvValue::I64(19066),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(127601175),
            TenxCsvValue::I64(19262),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(130149341),
            TenxCsvValue::I64(19279),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(129448959),
            TenxCsvValue::I64(19157),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(127618747),
            TenxCsvValue::I64(19955),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(130384916),
            TenxCsvValue::I64(19999),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(127979209),
            TenxCsvValue::I64(20074),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(135438861),
            TenxCsvValue::I64(19893),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(132257387),
            TenxCsvValue::I64(20070),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(128906123),
            TenxCsvValue::I64(19516),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(130424065),
            TenxCsvValue::I64(19286),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(126224136),
            TenxCsvValue::I64(19446),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(129136426),
            TenxCsvValue::I64(19852),
            TenxCsvValue::String("320k_K562_Flex_CRISPR_Ultima".to_owned()),
            TenxCsvValue::I64(132819446),
        ];

        assert_eq!(parsed_data, expected_data);
    }

    #[test]
    fn extract_from_str_with_parentheses() {
        let Some(extracted_str) = extract_numeric_part("312,195 (100.0%)") else {
            panic!("expected to extract number");
        };

        assert_eq!(extracted_str, "312,195")
    }

    #[test]
    fn extract_from_str_without_parentheses() {
        let Some(extracted_str) = extract_numeric_part("312,195") else {
            panic!("expected to extract number");
        };

        assert_eq!(extracted_str, "312,195")
    }

    #[test]
    fn extract_from_percentage() {
        let Some(extracted_str) = extract_numeric_part("92.6%") else {
            panic!("expected to extract number");
        };

        assert_eq!(extracted_str, "92.6%")
    }
}
