//! Memory the process can actually give back.
//!
//! The gate's peak is not a live-data measurement. On 2026-09-07 a probe
//! parsed three captures in one process and watched the resident set:
//! after `drop(log)` and `drop(text)` it had not fallen by a byte — 5,332
//! MiB still held with nothing alive. Every large block here goes through
//! the system allocator, and macOS's allocator keeps a freed block of that
//! size rather than unmapping it, so a suite that parses forty captures one
//! after another **ratchets**: the peak `tools/memcap.sh` reports is the sum
//! of everything the process ever held, not the most it ever needed.
//!
//! So the two large things a parse holds — the capture's own text, and the
//! arena's chunks — are mapped rather than allocated. A mapping is returned
//! by `munmap` at the moment it is dropped, with no allocator in between,
//! which is what turns the gate's number back into a measurement of live
//! data (item 260).
//!
//! What a mapping does **not** buy on this machine is a scan that runs
//! without the file resident. The pages are file-backed and clean, so a
//! walk that has finished with a frame ought to be able to hand them back;
//! but macOS accepts `madvise(MADV_DONTNEED)` over a private file mapping,
//! answers 0, and leaves the resident set exactly where it was, and
//! `MADV_FREE_REUSABLE` — the one that does move it — refuses anything but
//! anonymous memory with `EINVAL` (measured 2026-09-07 on a 1.3 GB
//! capture: 1,347 MiB resident before the call and after it). So a
//! capture's text is resident for as long as it is mapped, and the peak
//! `tools/memcap.sh` reports counts it. It is *clean* — the kernel would
//! evict it under real pressure rather than swap it — which is why a
//! machine with 128 GB never noticed; the meter counts it all the same.
//!
//! This module is the one place in `crates/rondata` where `unsafe` is
//! allowed: a mapping cannot be made in safe Rust and no crate hides it
//! (`memmap2::Mmap::map` is itself an `unsafe fn`). The crate's
//! `Cargo.toml` says the same from the other side.
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ops::Deref;
use std::path::Path;

#[cfg(unix)]
mod sys {
    use std::ffi::c_void;

    unsafe extern "C" {
        pub fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: i32,
            flags: i32,
            fd: i32,
            offset: i64,
        ) -> *mut c_void;
        pub fn munmap(addr: *mut c_void, len: usize) -> i32;
    }

    pub const PROT_READ: i32 = 0x1;
    pub const PROT_WRITE: i32 = 0x2;
    pub const MAP_PRIVATE: i32 = 0x0002;
    // The one constant the two kernels disagree on.
    #[cfg(target_os = "linux")]
    pub const MAP_ANON: i32 = 0x20;
    #[cfg(not(target_os = "linux"))]
    pub const MAP_ANON: i32 = 0x1000;

    pub fn failed(p: *mut c_void) -> bool {
        p as isize == -1
    }
}

/// A whole capture's text, mapped rather than read.
///
/// Derefs to `str`, so `Log::parse(&text)` takes it exactly as it took a
/// `String`. A file that is empty, unmappable, or not UTF-8 falls back to
/// an owned `String`, so the type is a drop-in everywhere.
pub struct Text {
    owned: Option<String>,
    ptr: *mut c_void,
    len: usize,
}

// The mapping is this value's alone: nothing else holds the pointer, and
// the only thing done to it is reading bytes.
unsafe impl Send for Text {}
unsafe impl Sync for Text {}

impl Text {
    /// The empty text — what an unreadable path answers with.
    pub fn empty() -> Text {
        Text {
            owned: Some(String::new()),
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }

    /// Wraps an owned string, for a caller that already has one.
    pub fn from_string(s: String) -> Text {
        Text {
            owned: Some(s),
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }
}

impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        match &self.owned {
            Some(s) => s,
            // SAFETY: the mapping is `self.len` readable bytes that
            // `read` has already checked are UTF-8, and it lives as long
            // as `self` does.
            None => unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.ptr as *const u8,
                    self.len,
                ))
            },
        }
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.owned.is_none() && !self.ptr.is_null() {
            // SAFETY: the mapping is this value's own and nothing borrows
            // it any more.
            unsafe { sys::munmap(self.ptr, self.len) };
        }
    }
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Text({} bytes)", self.len())
    }
}

/// Maps a capture. An unreadable path answers with the empty text — the
/// same shape `std::fs::read_to_string(..).unwrap_or_default()` has, and
/// every caller here has already checked the file is there.
pub fn read(path: impl AsRef<Path>) -> Text {
    read_inner(path.as_ref())
}

#[cfg(unix)]
fn read_inner(path: &Path) -> Text {
    use std::os::unix::io::AsRawFd;

    let Ok(file) = std::fs::File::open(path) else {
        return Text::empty();
    };
    let len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(_) => return Text::empty(),
    };
    if len == 0 {
        return Text::empty();
    }
    // SAFETY: a read-only private mapping of an open file; the pointer is
    // checked against `MAP_FAILED` before it is used.
    let ptr = unsafe {
        sys::mmap(
            std::ptr::null_mut(),
            len,
            sys::PROT_READ,
            sys::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
    };
    if sys::failed(ptr) {
        return std::fs::read_to_string(path).map_or_else(|_| Text::empty(), Text::from_string);
    }
    let mapped = Text {
        owned: None,
        ptr,
        len,
    };
    // SAFETY: `mapped` owns `len` readable bytes at `ptr`.
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    if std::str::from_utf8(bytes).is_err() {
        drop(mapped);
        return std::fs::read_to_string(path).map_or_else(|_| Text::empty(), Text::from_string);
    }
    mapped
}

#[cfg(not(unix))]
fn read_inner(path: &Path) -> Text {
    std::fs::read_to_string(path).map_or_else(|_| Text::empty(), Text::from_string)
}

/// A fixed-length run of `T`, in a mapping of its own.
///
/// The arena's unit of growth. A `Box<[T]>` of this size is a large
/// allocation the system allocator caches rather than returns; a mapping
/// goes back to the kernel the moment it is dropped, which is the whole
/// point (see the module note).
pub(crate) struct Pages<T> {
    ptr: *mut T,
    len: usize,
    /// False on the boxed-slice fallback, which is freed rather than
    /// unmapped. Nothing but a failed `mmap` takes that path.
    mapped: bool,
}

// SAFETY: the mapping is this value's alone, so it is as sendable and as
// shareable as the `T`s it holds.
unsafe impl<T: Send> Send for Pages<T> {}
unsafe impl<T: Sync> Sync for Pages<T> {}

impl<T: Copy + Default> Pages<T> {
    /// `len` entries, every one of them `T::default()`.
    pub(crate) fn new(len: usize) -> Pages<T> {
        let bytes = len * size_of::<T>();
        #[cfg(unix)]
        {
            // SAFETY: an anonymous read-write mapping; checked against
            // `MAP_FAILED` before anything is written through it.
            let ptr = unsafe {
                sys::mmap(
                    std::ptr::null_mut(),
                    bytes,
                    sys::PROT_READ | sys::PROT_WRITE,
                    sys::MAP_PRIVATE | sys::MAP_ANON,
                    -1,
                    0,
                )
            };
            if !sys::failed(ptr) {
                let ptr = ptr as *mut T;
                for i in 0..len {
                    // SAFETY: `i < len`, and the mapping is writable and
                    // aligned to a page, so to anything this holds.
                    unsafe { ptr.add(i).write(T::default()) };
                }
                return Pages {
                    ptr,
                    len,
                    mapped: true,
                };
            }
        }
        let mut v = vec![T::default(); len].into_boxed_slice();
        let ptr = v.as_mut_ptr();
        std::mem::forget(v);
        Pages {
            ptr,
            len,
            mapped: false,
        }
    }
}

impl<T> Deref for Pages<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        // SAFETY: `len` initialised entries at `ptr`, owned by `self`.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T> std::ops::DerefMut for Pages<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: as above, and `&mut self` makes the borrow unique.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T: Copy + Default> Clone for Pages<T> {
    fn clone(&self) -> Pages<T> {
        let mut p = Pages::new(self.len);
        p.copy_from_slice(self);
        p
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Pages<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Drop for Pages<T> {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.mapped {
            // SAFETY: the mapping is this value's own and nothing borrows
            // it any more. `T` here is always `Copy`, so there is nothing
            // to run a destructor for.
            unsafe { sys::munmap(self.ptr as *mut c_void, self.len * size_of::<T>()) };
            return;
        }
        // SAFETY: the fallback path built this from a boxed slice of
        // exactly `len` entries and forgot it.
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                self.ptr, self.len,
            )))
        };
    }
}
