use std::cmp::min;
use byteorder::ByteOrder;
use serde::Deserialize;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum HexEncoding {
    Byte,
    Little32,
    Big32
}

fn read_with_encoding<T: Into<u64>, F: FnMut(&[u8]) -> T>(data: &[u8], mut f: F) -> Vec<u64> {
    (0 .. data.len()).step_by(4)
        .map(|i| {
            let slice = &data[i .. min(i + 4, data.len())];

            f(slice).into()
        }).collect()
}

pub fn encode_hex_with_encoding(data: &[u8], encoding: HexEncoding) -> String {
    let result: Vec<u64> = match encoding {
        HexEncoding::Byte => data.iter().cloned().map(|x| x as u64).collect(),
        HexEncoding::Little32 => read_with_encoding(data, byteorder::LittleEndian::read_u32),
        HexEncoding::Big32 => read_with_encoding(data, byteorder::BigEndian::read_u32)
    };

    encode_hex(&result)
}

pub fn encode_hex(data: &[u64]) -> String {
    let leading = "v3.0 hex words plain";

    let items_per_line = 16;

    let body = (0 .. data.len()).step_by(items_per_line)
        .map(|global| {
            (0 .. items_per_line)
                .filter_map(|sub| data.get(global + sub))
                .cloned()
                .map(|value| {
                    if value < 0x100 {
                        format!("{:02x}", value)
                    } else if value < 0x10000 {
                        format!("{:04x}", value)
                    } else if value < 0x100000000 {
                        format!("{:08x}", value)
                    } else {
                        format!("{:016x}", value)
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!("{leading}\n{body}")
}
