pub fn encode_hex(data: Vec<u64>) -> String {
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
