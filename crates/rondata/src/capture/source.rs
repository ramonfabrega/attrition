//! Physical files or explicitly named experimental archives. No auto-discovery.
use super::archive::Archive;
use std::fs::{File, Metadata};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

pub(super) enum Source {
    Plain(File),
    Archive(Archive),
}
impl Source {
    pub(super) fn open(path: &Path) -> io::Result<Self> {
        if path.extension().is_some_and(|s| s == "rcap") {
            Ok(Self::Archive(Archive::open(path)?))
        } else {
            Ok(Self::Plain(File::open(path)?))
        }
    }
    pub(super) fn metadata(&self) -> io::Result<Metadata> {
        match self {
            Self::Plain(f) => f.metadata(),
            Self::Archive(a) => a.file.metadata(),
        }
    }
    pub(super) fn length(&self) -> io::Result<u64> {
        match self {
            Self::Plain(f) => Ok(f.metadata()?.len()),
            Self::Archive(a) => Ok(a.length()),
        }
    }
}
impl Read for Source {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(f) => f.read(out),
            Self::Archive(a) => a.read(out),
        }
    }
}
impl Seek for Source {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        match self {
            Self::Plain(f) => f.seek(from),
            Self::Archive(a) => a.seek(from),
        }
    }
}
