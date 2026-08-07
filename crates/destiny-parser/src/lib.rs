//! Conservative, filename-independent inspection of extracted Tiger definitions.
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signature {
    Seg,
    Definition8080,
    Text,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub kind: String,
    pub offset: u64,
    pub size: u64,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub signature: Signature,
    pub confidence: u8,
    pub hash: String,
    pub size: u64,
    pub fields: Vec<Field>,
    pub strings: Vec<(u64, String)>,
    pub references: Vec<(u64, String)>,
    pub tag_identifiers: Vec<(u64, String)>,
    pub type_information: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_file(path: &Path) -> Result<ParsedFile, ParseError> {
    let bytes = std::fs::read(path)?;
    Ok(parse_bytes(&bytes))
}

pub fn parse_bytes(bytes: &[u8]) -> ParsedFile {
    let hash = hex::encode(Sha256::digest(bytes));
    let (signature, confidence) = detect_signature(bytes);
    let mut fields = Vec::new();
    let mut warnings = Vec::new();
    // Common little-endian header candidates are recorded, never trusted as a schema.
    for (i, chunk) in bytes.chunks_exact(4).take(16).enumerate() {
        let value = u32::from_le_bytes(chunk.try_into().unwrap()) as u64;
        if value < bytes.len() as u64 {
            fields.push(Field {
                kind: "header_candidate".into(),
                offset: (i * 4) as u64,
                size: 4,
                value: Some(value.to_string()),
            });
        }
    }
    let strings = extract_strings(bytes);
    let mut tag_identifiers = Vec::new();
    for (offset, chunk) in bytes.chunks_exact(4).enumerate() {
        let value = u32::from_be_bytes(chunk.try_into().unwrap());
        if offset != 1 && is_tag_hash(value) {
            tag_identifiers.push(((offset * 4) as u64, format!("0x{value:08x}")));
        }
    }
    let type_information = if signature == Signature::Definition8080 && bytes.len() >= 8 {
        Some(format!(
            "0x{:08x}",
            u32::from_be_bytes(bytes[4..8].try_into().unwrap())
        ))
    } else {
        None
    };
    if signature == Signature::Unknown {
        warnings.push(
            "No known signature; retained hash, bounded header candidates, and strings".into(),
        );
    }
    if strings.is_empty() && !bytes.is_empty() {
        warnings.push("No printable string table found".into());
    }
    let mut references: Vec<(u64, String)> = strings
        .iter()
        .filter(|(_, s)| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .cloned()
        .collect();
    for (offset, ref_id) in &tag_identifiers {
        if *offset > 8 {
            references.push((*offset, ref_id.clone()));
        }
    }
    ParsedFile {
        signature,
        confidence,
        hash,
        size: bytes.len() as u64,
        fields,
        strings,
        references,
        tag_identifiers,
        type_information,
        warnings,
    }
}

fn detect_signature(b: &[u8]) -> (Signature, u8) {
    if b.len() >= 4 && (&b[..4] == b"SEGF" || &b[..4] == b"SEG\0") {
        return (Signature::Seg, 100);
    }
    // Extracted 8080/tag payloads have no ASCII magic. They conventionally
    // carry a big-endian byte length followed by a type/class word and one or
    // more 0x808xxxxx tag hashes. Validate all three signals before claiming it.
    if b.len() >= 16 {
        let declared = u32::from_be_bytes(b[..4].try_into().unwrap()) as usize;
        let has_tag = b
            .chunks_exact(4)
            .take(16)
            .enumerate()
            .any(|(i, c)| i != 1 && is_tag_hash(u32::from_be_bytes(c.try_into().unwrap())));
        if declared == b.len() && has_tag {
            return (Signature::Definition8080, 100);
        }
    }
    if !b.is_empty()
        && b.iter()
            .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
    {
        return (Signature::Text, 70);
    }
    (Signature::Unknown, 0)
}

fn is_tag_hash(value: u32) -> bool {
    (0x80800001..=0x81ffffff).contains(&value)
}

fn extract_strings(b: &[u8]) -> Vec<(u64, String)> {
    let mut out = Vec::new();
    let mut start = None;
    for (i, c) in b
        .iter()
        .copied()
        .enumerate()
        .chain(std::iter::once((b.len(), 0)))
    {
        let printable = c.is_ascii_graphic() || c == b' ' || c == b'\t';
        if printable && start.is_none() {
            start = Some(i);
        }
        if !printable {
            if let Some(s) = start.take() {
                if i - s >= 4 {
                    out.push((s as u64, String::from_utf8_lossy(&b[s..i]).into_owned()));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_content_not_extension() {
        assert_eq!(parse_bytes(b"SEGF\x01abc").signature, Signature::Seg);
    }
    #[test]
    fn detects_binary_8080_payload() {
        let mut b = vec![0u8; 20];
        b[0..4].copy_from_slice(&(20u32.to_be_bytes()));
        b[4..8].copy_from_slice(&0x81060d49u32.to_be_bytes());
        b[8..12].copy_from_slice(&0x80802797u32.to_be_bytes());
        let p = parse_bytes(&b);
        assert_eq!(p.signature, Signature::Definition8080);
        assert_eq!(p.tag_identifiers.len(), 1);
        assert_eq!(p.type_information.as_deref(), Some("0x81060d49"));
    }
    #[test]
    fn unknown_is_safe() {
        let p = parse_bytes(&[0, 1, 2, 3]);
        assert_eq!(p.signature, Signature::Unknown);
        assert!(!p.hash.is_empty());
    }
    #[test]
    fn extracts_strings() {
        assert_eq!(
            parse_bytes(b"xx hello world\0").strings[0].1,
            "xx hello world"
        );
    }
}
