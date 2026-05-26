# csvranger
A Rust library for parsing the output CSV files of [10x Genomics *ranger pipelines](https://www.10xgenomics.com/software).

Many of 10x Genomics data-processing pipelines produce CSV files that summarize the data. These CSVs, while human-readable, cannot be parsed by a machine without extra effort. This small crate provides the necessary functionality to parse the values in these files.
## Example usage
```rust
// The easiest entrypoint is TenxCsvValue::from_csv_value in conjunction with the csv crate
use csvranger::TenxCsvValue;

fn main() {
    // Sample data in test-data/cellranger_multi.10.0
    let raw_csv = b"Sample ID,Sample barcodes,Sample description,GEX: Cells,GEX: Confidently mapped reads in cells,GEX: Median UMI counts per cell,GEX: Median genes per cell,GEX: Total genes detected
    SOD1_G93A_mouse_spinal_cord_P112_specimen_1,,SOD1-G93A mouse spinal cord from P112 mouse,16410,0.6847850295990157,2244,1305,27219";

    let mut reader = csv::Reader::from_reader(raw_csv);
    for line in reader.records() {
        let line = line.unwrap();
        for value in line.iter() {
            // This is where the magic happens
            TenxCsvValue::from_csv_value(value);
        }
    } 
}
```

## Features
- **`legacy`**: If you are parsing CSV-files from [legacy 10x Genomics pipelines](#legacy-pipelines), you'll need to activate this feature.
- **`serde`**: Support for serializing a `TenxCsvValue` using [serde](https://serde.rs). Note that deserialization support is not currently implemented because the correct behavior is unclear - should the type parse as its underlying type, or should it use the parsing functionality? If you disagree, feel free to open an issue!
- **`schemars`**: Support for [`schemars`](https://docs.rs/schemars/latest/schemars/)

You can add features to your project directly to your `Cargo.toml`:
```toml
csvranger = { version = "0.1.0", features = ["legacy"] }
```
or using cargo:
```bash
cargo add csvranger --features legacy
```
## Tested pipelines
The outputs of the following pipeline-version combinations are tested, though others likely work as well:
- cellranger 6
- cellranger 8
- cellranger 9
- cellranger 10
- celranger-atac 2
- spaceranger 4

More pipeline-version combinations will be added.
## Legacy pipelines
If you are parsing CSVs produced by `cellranger <= 10`, you'll want to activate the `legacy` feature. The outputs of `cellranger-atac < 2` and `spaceranger < 4` are untested, but they likely do not require the `legacy` feature.
