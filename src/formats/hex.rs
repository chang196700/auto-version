use anyhow::{Context, Result};

/// Format major/minor/patch into a HEX value using a configurable bit layout.
///
/// Layout string: `"major:8,minor:8,patch:16"` means major uses bits[31:24],
/// minor bits[23:16], patch bits[15:0]. Total must not exceed 64 bits.
pub fn format_hex(major: u64, minor: u64, patch: u64, layout: &str, prefix: &str) -> Result<String> {
    let parts = parse_layout(layout)?;
    let mut value: u64 = 0;
    for (component, bits) in &parts {
        let component_val = match component.as_str() {
            "major" => major,
            "minor" => minor,
            "patch" => patch,
            other   => anyhow::bail!("unknown component in hex layout: {}", other),
        };
        let mask = if *bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 };
        value = (value << bits) | (component_val & mask);
    }

    let total_bits: u32 = parts.iter().map(|(_, b)| b).sum();
    let hex_digits = ((total_bits + 3) / 4) as usize;
    Ok(format!("{}{:0>width$X}", prefix, value, width = hex_digits))
}

fn parse_layout(layout: &str) -> Result<Vec<(String, u32)>> {
    layout
        .split(',')
        .map(|part| {
            let (name, bits) = part
                .split_once(':')
                .with_context(|| format!("invalid hex layout segment: '{}'", part))?;
            let bits: u32 = bits
                .trim()
                .parse()
                .with_context(|| format!("parsing bit count in layout segment: '{}'", part))?;
            Ok((name.trim().to_string(), bits))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_format_default_layout() {
        // major:8, minor:8, patch:16 → 32-bit value
        let hex = format_hex(1, 2, 3, "major:8,minor:8,patch:16", "0x").unwrap();
        assert_eq!(hex, "0x01020003");
    }

    #[test]
    fn test_hex_format_custom_layout() {
        // major:4, minor:4, patch:8 → 16-bit value
        let hex = format_hex(1, 2, 5, "major:4,minor:4,patch:8", "0x").unwrap();
        assert_eq!(hex, "0x1205");
    }
}
