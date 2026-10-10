//! Opt-in experimental seekable capture archive. Original captures stay intact.
//! Independent gzip members or checksummed LZ4 blocks bound reads to 4 MiB chunks.
//! Two decoded chunks (8 MiB plus at most two overflow-detection bytes) are cached.
//! This is a local format, not a claim about the game's formats.
use flate2::{Compression, Crc, bufread::GzDecoder, write::GzEncoder};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"RONCAP01";
const END: &[u8; 8] = b"ENDCAP01";
const LZ4_MAGIC: &[u8; 8] = b"RONCAP02";
const LZ4_END: &[u8; 8] = b"ENDCAP02";

/// Explicit archive codec. Gzip keeps the original trial format readable;
/// LZ4 uses safe Rust encode/decode plus a CRC32 for each original chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    Gzip,
    Lz4,
}
impl Codec {
    fn magic(self) -> &'static [u8; 8] {
        match self {
            Self::Gzip => MAGIC,
            Self::Lz4 => LZ4_MAGIC,
        }
    }
    fn end(self) -> &'static [u8; 8] {
        match self {
            Self::Gzip => END,
            Self::Lz4 => LZ4_END,
        }
    }
    fn minimum_chunk(self) -> u32 {
        match self {
            Self::Gzip => 18,
            Self::Lz4 => 1,
        }
    }
}
const CHUNK: usize = 4 * 1024 * 1024;
const HEADER: u64 = 24;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn checksum(bytes: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.update(bytes);
    crc.sum()
}

pub(crate) struct Archive {
    pub(crate) file: File,
    length: u64,
    chunks: Vec<(u64, u32, Option<u32>)>,
    position: u64,
    cache: VecDeque<(usize, Vec<u8>)>,
}
impl Archive {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let stored = file.metadata()?.len();
        let mut header = [0; HEADER as usize];
        file.read_exact(&mut header)?;
        let codec = match &header[..8] {
            bytes if bytes == MAGIC => Codec::Gzip,
            bytes if bytes == LZ4_MAGIC => Codec::Lz4,
            _ => return Err(invalid("unsupported capture archive version")),
        };
        if u32::from_le_bytes(header[8..12].try_into().unwrap()) != CHUNK as u32
            || checksum(&header[..20]) != u32::from_le_bytes(header[20..].try_into().unwrap())
        {
            return Err(invalid("invalid capture archive header"));
        }
        let length = u64::from_le_bytes(header[12..20].try_into().unwrap());
        let count = length.div_ceil(CHUNK as u64);
        // Reject a forged size before allocating an index or iterating counts.
        let overhead = if codec == Codec::Lz4 { 8 } else { 4 };
        if count
            > stored.saturating_sub(HEADER + 12) / (u64::from(codec.minimum_chunk()) + overhead)
        {
            return Err(invalid("archive chunk count exceeds file"));
        }
        let mut crc = Crc::new();
        crc.update(&header);
        let mut chunks = Vec::new();
        let mut offset = HEADER;
        for _ in 0..count {
            file.seek(SeekFrom::Start(offset))?;
            let mut raw = [0; 4];
            file.read_exact(&mut raw)?;
            crc.update(&raw);
            let size = u32::from_le_bytes(raw);
            if !(codec.minimum_chunk()..=CHUNK as u32 + 65536).contains(&size) {
                return Err(invalid("invalid compressed chunk size"));
            }
            offset += 4;
            let decoded_crc = if codec == Codec::Lz4 {
                file.read_exact(&mut raw)?;
                crc.update(&raw);
                offset += 4;
                Some(u32::from_le_bytes(raw))
            } else {
                None
            };
            chunks.push((offset, size, decoded_crc));
            offset += u64::from(size);
            if offset > stored.saturating_sub(12) {
                return Err(invalid("truncated capture archive"));
            }
        }
        if offset + 12 != stored {
            return Err(invalid("archive has missing footer or trailing bytes"));
        }
        file.seek(SeekFrom::Start(offset))?;
        let mut footer = [0; 12];
        file.read_exact(&mut footer)?;
        if &footer[..8] != codec.end()
            || u32::from_le_bytes(footer[8..].try_into().unwrap()) != crc.sum()
        {
            return Err(invalid(
                "invalid archive index checksum or publication footer",
            ));
        }
        Ok(Self {
            file,
            length,
            chunks,
            position: 0,
            cache: VecDeque::new(),
        })
    }

    pub(crate) fn length(&self) -> u64 {
        self.length
    }

    fn load(&mut self, index: usize) -> io::Result<()> {
        if let Some(at) = self.cache.iter().position(|(i, _)| *i == index) {
            let hit = self.cache.remove(at).unwrap();
            self.cache.push_back(hit);
            return Ok(());
        }
        let (offset, size, decoded_crc) = self.chunks[index];
        let expected = (self.length - index as u64 * CHUNK as u64).min(CHUNK as u64);
        self.file.seek(SeekFrom::Start(offset))?;
        let mut compressed = vec![0; size as usize];
        self.file.read_exact(&mut compressed)?;
        // Two chunks keep a boundary-spanning frame's second consumer from
        // immediately decompressing both chunks again. Reuse the evicted buffer.
        let mut bytes = if self.cache.len() == 2 {
            self.cache.pop_front().unwrap().1
        } else {
            Vec::with_capacity(CHUNK + 1)
        };
        if let Some(crc) = decoded_crc {
            bytes.resize(expected as usize, 0);
            let written = lz4_flex::block::decompress_into(&compressed, &mut bytes)
                .map_err(|_| invalid("invalid LZ4 block"))?;
            if written != expected as usize || checksum(&bytes) != crc {
                return Err(invalid("LZ4 chunk length or checksum mismatch"));
            }
        } else {
            let mut decoder = GzDecoder::new(compressed.as_slice());
            // One extra byte detects expansion past the advertised size; reading
            // to EOF checks the gzip trailer/CRC, even for a full-size chunk. Read
            // into fixed spare space so Vec growth cannot double the cache budget.
            bytes.resize(expected as usize + 1, 0);
            let mut filled = 0;
            while filled < bytes.len() {
                let n = decoder.read(&mut bytes[filled..])?;
                if n == 0 {
                    break;
                }
                filled += n;
            }
            bytes.truncate(filled);
            if bytes.len() as u64 != expected || !decoder.get_ref().is_empty() {
                return Err(invalid("archive chunk length or member boundary mismatch"));
            }
        }
        self.cache.push_back((index, bytes));
        Ok(())
    }
}
impl Read for Archive {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.length || out.is_empty() {
            return Ok(0);
        }
        self.load((self.position / CHUNK as u64) as usize)?;
        let start = (self.position % CHUNK as u64) as usize;
        let bytes = &self.cache.back().expect("loaded chunk").1;
        let size = out.len().min(bytes.len() - start);
        out[..size].copy_from_slice(&bytes[start..start + size]);
        self.position += size as u64;
        Ok(size)
    }
}
impl Seek for Archive {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::End(n) => i128::from(self.length) + i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
        };
        self.position = u64::try_from(position)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid archive seek"))?;
        Ok(self.position)
    }
}

/// Compress a finalized source to a new archive. Writes a create-new sibling
/// `.partial`, syncs and verifies every byte, then publishes with a no-clobber
/// hard link. An interrupted/failed conversion leaves its partial for inspection;
/// readers never discover it automatically. Neither existing path is replaced.
/// The destination directory must already exist on a filesystem with hard links.
pub fn pack(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    pack_with_codec(source, destination, Codec::Gzip)
}

/// [`pack`] with an explicit codec; verification and publication are identical.
pub fn pack_with_codec(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    codec: Codec,
) -> io::Result<()> {
    let destination = destination.as_ref();
    if destination.extension().is_none_or(|s| s != "rcap") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination must end in .rcap",
        ));
    }
    if destination.try_exists()? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "archive exists",
        ));
    }
    let source = source.as_ref();
    let mut input = File::open(source)?;
    let before = input.metadata()?;
    let partial = destination.with_extension("rcap.partial");
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)?;
    let mut header = Vec::from(codec.magic().as_slice());
    header.extend_from_slice(&(CHUNK as u32).to_le_bytes());
    header.extend_from_slice(&before.len().to_le_bytes());
    header.extend_from_slice(&checksum(&header).to_le_bytes());
    output.write_all(&header)?;
    let mut crc = Crc::new();
    crc.update(&header);
    let mut remaining = before.len();
    let mut bytes = vec![0; CHUNK];
    while remaining != 0 {
        let size = remaining.min(CHUNK as u64) as usize;
        input.read_exact(&mut bytes[..size])?;
        let compressed = match codec {
            Codec::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
                encoder.write_all(&bytes[..size])?;
                encoder.finish()?
            }
            Codec::Lz4 => lz4_flex::block::compress(&bytes[..size]),
        };
        let size_bytes = u32::try_from(compressed.len())
            .map_err(|_| invalid("compressed chunk too large"))?
            .to_le_bytes();
        crc.update(&size_bytes);
        output.write_all(&size_bytes)?;
        if codec == Codec::Lz4 {
            let decoded_crc = checksum(&bytes[..size]).to_le_bytes();
            crc.update(&decoded_crc);
            output.write_all(&decoded_crc)?;
        }
        output.write_all(&compressed)?;
        remaining -= size as u64;
    }
    output.write_all(codec.end())?;
    output.write_all(&crc.sum().to_le_bytes())?;
    output.sync_all()?;
    drop(output);
    let mut archive = Archive::open(&partial)?;
    input.rewind()?;
    let mut decoded = vec![0; CHUNK];
    remaining = before.len();
    while remaining != 0 {
        let size = remaining.min(CHUNK as u64) as usize;
        input.read_exact(&mut bytes[..size])?;
        archive.read_exact(&mut decoded[..size])?;
        if bytes[..size] != decoded[..size] {
            return Err(invalid("archive roundtrip differs"));
        }
        remaining -= size as u64;
    }
    for after in [input.metadata()?, std::fs::metadata(source)?] {
        if before.len() != after.len() || before.modified()? != after.modified()? {
            return Err(invalid("source changed during compression"));
        }
    }
    drop(archive);
    std::fs::hard_link(&partial, destination)?;
    // Only our newly created, now-published temporary link is removed.
    std::fs::remove_file(&partial)?;
    File::open(
        destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?
    .sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::indexed::IndexedCapture;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ron-archive-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
        fn packed_with(&self, bytes: &[u8], codec: Codec) -> PathBuf {
            let source = self.path("source.txt");
            let archive = self.path("capture.rcap");
            std::fs::write(&source, bytes).unwrap();
            pack_with_codec(source, &archive, codec).unwrap();
            archive
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn byte_roundtrip_seeks_and_chunk_boundaries() {
        for codec in [Codec::Gzip, Codec::Lz4] {
            for length in [0, 1, CHUNK - 1, CHUNK, CHUNK + 1, CHUNK * 2 + 83] {
                let scratch = Scratch::new();
                let bytes: Vec<u8> = (0..length)
                    .map(|i| ((i * 31 + i / 251) % 256) as u8)
                    .collect();
                let path = scratch.packed_with(&bytes, codec);
                let mut archive = Archive::open(&path).unwrap();
                let mut all = Vec::new();
                archive.read_to_end(&mut all).unwrap();
                assert_eq!(all, bytes);
                for position in [length, length / 2, 0, CHUNK - 1, 1, CHUNK + 1] {
                    archive.seek(SeekFrom::Start(position as u64)).unwrap();
                    let mut got = vec![0; 83.min(length.saturating_sub(position))];
                    archive.read_exact(&mut got).unwrap();
                    assert!(archive.cache.len() <= 2);
                    assert!(
                        archive
                            .cache
                            .iter()
                            .map(|(_, b)| b.capacity())
                            .sum::<usize>()
                            <= 2 * (CHUNK + 1)
                    );
                    if position <= length {
                        assert_eq!(got, bytes[position..position + got.len()]);
                    }
                }
                assert!(archive.seek(SeekFrom::End(-(length as i64) - 1)).is_err());
            }
        }
    }

    #[test]
    fn indexed_raw_and_archive_preserve_duplicates_setup_and_siblings() {
        for codec in [Codec::Gzip, Codec::Lz4] {
            let scratch = Scratch::new();
            let mut text = String::from("BEGIN GAME\n seed 12\n BEGIN FRAME 7\n  x 1\n");
            // Split both a line and a frame across compression chunk boundaries.
            text.push_str(&" ".repeat(CHUNK));
            text.push_str("\n BEGIN FULL DUMP\n  who 1\n BEGIN FRAME 7\n  x 2\n BEGIN FRAME 9\n  x 3\nBEGIN GAME INFO\n winner 1\n");
            let path = scratch.packed_with(text.as_bytes(), codec);
            let mut raw = IndexedCapture::open(scratch.path("source.txt")).unwrap();
            let mut zipped = IndexedCapture::open(&path).unwrap();
            assert_eq!(raw.source_bytes(), zipped.source_bytes());
            assert_eq!(raw.frames(), zipped.frames());
            assert_eq!(
                raw.read_replay_setup().unwrap(),
                zipped.read_replay_setup().unwrap()
            );
            assert_eq!(
                raw.read_shutdown().unwrap(),
                zipped.read_shutdown().unwrap()
            );
            for i in [2, 0, 1, 0, 2] {
                assert_eq!(raw.read_frame(i).unwrap(), zipped.read_frame(i).unwrap());
                assert_eq!(
                    raw.read_frame_and_siblings(i).unwrap(),
                    zipped.read_frame_and_siblings(i).unwrap()
                );
            }
            assert!(zipped.read_frame(3).is_err());
            OpenOptions::new()
                .append(true)
                .open(path)
                .unwrap()
                .write_all(b"x")
                .unwrap();
            assert!(
                zipped.read_frame(0).is_err(),
                "changed archive must invalidate cached chunks/index"
            );
        }
    }

    #[test]
    fn truncation_corruption_and_trailing_bytes_fail_closed() {
        for codec in [Codec::Gzip, Codec::Lz4] {
            let scratch = Scratch::new();
            let path = scratch.packed_with(&vec![b'x'; CHUNK + 17], codec);
            let original = std::fs::read(&path).unwrap();
            let chunks = Archive::open(&path).unwrap().chunks;
            let mut variants = Vec::new();
            for length in [
                0,
                8,
                20,
                24,
                original.len() / 2,
                original.len() - 12,
                original.len() - 1,
            ] {
                variants.push(original[..length].to_vec());
            }
            for offset in [
                0,
                8,
                12,
                20,
                24,
                chunks[0].0 as usize + 10,
                (chunks[0].0 + u64::from(chunks[0].1) - 8) as usize,
                original.len() - 1,
            ] {
                let mut bytes = original.clone();
                bytes[offset] ^= 1;
                variants.push(bytes);
            }
            let mut extra = original.clone();
            extra.push(0);
            variants.push(extra);
            for bytes in variants {
                let bad = scratch.path("bad.rcap");
                std::fs::write(&bad, bytes).unwrap();
                let result = Archive::open(&bad).and_then(|mut a| a.read_to_end(&mut Vec::new()));
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn publication_never_overwrites_and_interrupted_partial_is_preserved() {
        for codec in [Codec::Gzip, Codec::Lz4] {
            let scratch = Scratch::new();
            let source = scratch.path("source.txt");
            std::fs::write(&source, b"complete source").unwrap();
            let dest = scratch.path("capture.rcap");
            let partial = dest.with_extension("rcap.partial");
            std::fs::write(&partial, b"interrupted writer").unwrap();
            assert_eq!(
                pack_with_codec(&source, &dest, codec).unwrap_err().kind(),
                io::ErrorKind::AlreadyExists
            );
            assert!(!dest.exists());
            assert_eq!(std::fs::read(&partial).unwrap(), b"interrupted writer");
            std::fs::remove_file(&partial).unwrap(); // this test's own disposable fixture
            pack_with_codec(&source, &dest, codec).unwrap();
            assert!(!partial.exists());
            let before = std::fs::read(&dest).unwrap();
            assert_eq!(
                pack_with_codec(&source, &dest, codec).unwrap_err().kind(),
                io::ErrorKind::AlreadyExists
            );
            assert_eq!(std::fs::read(&dest).unwrap(), before);
            assert_eq!(std::fs::read(&source).unwrap(), b"complete source");
        }
    }

    #[test]
    fn excessive_expansion_is_bounded_and_wrong_logical_length_is_rejected() {
        for codec in [Codec::Gzip, Codec::Lz4] {
            let scratch = Scratch::new();
            let path = scratch.packed_with(&vec![b'x'; CHUNK], codec);
            let mut bytes = std::fs::read(&path).unwrap();
            // A valid metadata checksum must not authorize the wrong decoded length.
            bytes[12..20].copy_from_slice(&1u64.to_le_bytes());
            let sum = checksum(&bytes[..20]);
            bytes[20..24].copy_from_slice(&sum.to_le_bytes());
            let metadata_end = if codec == Codec::Lz4 { 32 } else { 28 };
            let sum = checksum(&bytes[..metadata_end]);
            let end = bytes.len();
            bytes[end - 4..].copy_from_slice(&sum.to_le_bytes());
            std::fs::write(&path, bytes).unwrap();
            let mut archive = Archive::open(&path).unwrap();
            assert!(archive.read_to_end(&mut Vec::new()).is_err());
            assert!(
                archive.cache.is_empty(),
                "failed decode must never be cached"
            );
        }
    }

    #[test]
    fn lz4_payload_crc_is_checked_even_with_valid_metadata() {
        let scratch = Scratch::new();
        let path = scratch.packed_with(b"the exact original text", Codec::Lz4);
        let mut bytes = std::fs::read(&path).unwrap();
        // Forge a valid metadata envelope around an incorrect decoded CRC.
        bytes[28] ^= 1;
        let sum = checksum(&bytes[..32]);
        let end = bytes.len();
        bytes[end - 4..].copy_from_slice(&sum.to_le_bytes());
        std::fs::write(&path, bytes).unwrap();
        let mut archive = Archive::open(&path).unwrap();
        assert!(archive.read_to_end(&mut Vec::new()).is_err());
        assert!(archive.cache.is_empty());
    }
}
