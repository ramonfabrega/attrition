//! Synthetic evidence-integrity probe; no original trace content is embedded.
use rondata::trace::Trace;
fn main() {
    let words = [0x544e4f52u32, 2, 0x400000, 0x1000, 0x6c4000, 0, 0, 0];
    let header: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    let mut unknown = header.clone();
    unknown[4..8].copy_from_slice(&999u32.to_le_bytes());
    let mut truncated = header.clone();
    truncated.push(1);
    let mut loss = header.clone();
    for word in [5u32, 14, 123, 0, 0, 0, 0, 0] {
        loss.extend_from_slice(&word.to_le_bytes());
    }
    for (name, bytes) in [
        ("header_only", header),
        ("unknown_version", unknown),
        ("partial_record", truncated),
        ("123_dropped_records", loss),
    ] {
        let trace = Trace::parse(&bytes);
        println!("{name}: accepted={}", trace.is_some());
    }
}
