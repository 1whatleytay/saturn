use serde::{Deserialize, Serialize};
use titan::assembler::binary::{RawRegion, RegionFlags};
use crate::build::{assemble_text, AssemblerResult};
use crate::hex_format::{encode_hex_with_encoding, HexEncoding};
use titan::assembler::binary::Binary;
use base64::Engine;

#[derive(Serialize, Deserialize)]
pub struct HexRegion {
    pub name: String,
    pub data: String // base64 encoded
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AssembleRegionsKind {
    Plain,
    HexV3
}

#[derive(Serialize, Deserialize)]
pub struct AssembleRegionsOptions {
    pub kind: AssembleRegionsKind,
    pub continuous: bool,
    pub encoding: HexEncoding // Encoding option ignored if kind != HexV3
}

#[derive(Serialize)]
#[serde(rename_all="snake_case", tag="type", content="value")]
pub enum AssembledRegions {
    Binary(String), // base64 encoded
    Split(Vec<HexRegion>)
}

fn region_name(region: &RawRegion, entry: bool) -> String {
    let address = region.address;

    let heading = if entry {
        "entry"
    } else if region.flags.contains(RegionFlags::EXECUTABLE) {
        "code"
    } else {
        "data"
    };

    let flags = [
        (RegionFlags::EXECUTABLE, "x"),
        (RegionFlags::READABLE, "r"),
        (RegionFlags::WRITABLE, "w")
    ]
        .into_iter()
        .map(|(f, s)| {
            if region.flags.contains(f) { s } else { "o" }
        })
        .collect::<Vec<&str>>()
        .join("");

    format!("{heading}_{address:x}_{flags}")
}

fn encode_region_data(data: &[u8], options: &AssembleRegionsOptions) -> String {
    match options.kind {
        AssembleRegionsKind::HexV3 => {
            let encoding = encode_hex_with_encoding(data, options.encoding);

            base64::engine::general_purpose::STANDARD.encode(encoding)
        },
        AssembleRegionsKind::Plain => {
            base64::engine::general_purpose::STANDARD.encode(data)
        }
    }
}

pub fn export_continuous(binary: &Binary, options: &AssembleRegionsOptions) -> String {
    let mut output: Vec<u8> = vec![];

    for region in &binary.regions {
        if region.data.is_empty() {
            continue
        }

        // Potential Overflow!
        let end = region.address as usize + region.data.len();

        if end > output.len() {
            output.resize(end, 0);
        }

        output[region.address as usize .. end].copy_from_slice(&region.data);
    };

    encode_region_data(&output, options)
}

fn export_regions(binary: &Binary, options: &AssembleRegionsOptions) -> Vec<HexRegion> {
    binary.regions.iter().filter_map(|region| {
        if region.data.is_empty() {
            return None
        }

        let name = region_name(region, region.address == binary.entry);

        Some(HexRegion {
            name,
            data: encode_region_data(&region.data, options),
        })
    }).collect()
}

pub fn assemble_regions(
    text: &str, path: Option<&str>, options: AssembleRegionsOptions
) -> (Option<AssembledRegions>, AssemblerResult) {
    let result = assemble_text(text, path);
    let (binary, result) = AssemblerResult::from_result_with_binary(result, text);

    let Some(binary) = binary else {
        return (None, result)
    };

    let regions = if options.continuous {
        AssembledRegions::Binary(export_continuous(&binary, &options))
    } else {
        AssembledRegions::Split(export_regions(&binary, &options))
    };

    (Some(regions), result)
}
