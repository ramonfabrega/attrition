//! Reads the original's own per-frame state dump, `Logs\gamelog.txt`.
//!
//! With `AllowLogs=1` in `rise.ini` the 2003 engine's `Log` system writes a
//! nested text dump of whichever subsystems `gamelog.ini` enables — once at
//! start (`[Start Game]`, plus the `InitialDump=1` state) and then **every
//! frame** (`[End Frame]`). Each object writes itself through its own
//! `log_data` virtual. `docs/ORACLE.md` has how it is switched on and what it
//! costs; this module is what reads the result back.
//!
//! # The grammar, as observed
//!
//! There is no schema. The file is a flat sequence of lines, each indented by
//! one space per nesting level:
//!
//! ```text
//! BEGIN GAME
//!  BEGIN WORLD
//!   seed 7236
//!   xs 60
//!  BEGIN UNITDATA
//!   BEGIN OBJECT
//!    BEGIN SUBOBJECT
//!     flags 65
//!     o 0
//!   BEGIN GUY
//!    type 69
//! ```
//!
//! A `BEGIN <name>` line opens a block; there is no `END`. A block closes
//! when a later `BEGIN` appears at the same or a shallower indent. Any other
//! line is a field: the first token is the key, the rest is the value, and it
//! belongs to the **innermost open block regardless of its own indent** —
//! because the writers are not consistent about it. The leaders writer emits
//! each leader's `leader_flags`/`leader_flags2` one level shallower than the
//! `who`/`tribe` lines and **before** the `BEGIN LEADERDATA` they describe
//! (the first pair in a dump precedes the first block; the last block has
//! none after it), so indentation alone hands each block its *successor's*
//! flags — `records` re-zips them by position from the parent's ordered run
//! (2026-08-23, found when `leader_flags & 4` = `is_human` started to
//! matter). Keys repeat: an array constant is written as one
//! line per element, all under the same key (`fort_upgrade_terr[scan] 2`,
//! `… 4`, `… 6`, `… 9`), in index order. Lines before the first `BEGIN` — the
//! `init_teams:` chatter and the splash-screen timing — are preamble and are
//! kept as fields with no block.
//!
//! Everything borrows from the input text: a 114 MB initial dump parses into
//! a tree of slices rather than a tree of copies.
//!
//! # What the dump contains, at detail level 0
//!
//! Established by reading two runs of this install (`docs/ORACLE.md`, last
//! section). Per unit: the `Object` base — `flags`, the object number `o`,
//! the owner `who`, `x_internal`/`y_internal`/`z_internal` in position units
//! — and, **only in the start-of-game dump**, one `GUY` per member with its
//! `type` (the unit type id), position and `angle`; per frame the `GUY`
//! blocks are written empty. Per leader: `who`, `tribe`, `defeated_by`,
//! `gov`, `score`, `leader_flags`, `leader_flags2` — no goods. The richer
//! fields (`UnitData::log_data` goes on to fifty more) wait on a detail-level
//! argument that `DUMP_ALL=1` presumably raises; untried.
//!
//! The `CONSTANTS` block is the loaded `Constants` struct, field by field, in
//! memory representation, under the **lowercased `rules.xml` tag** — 718 of
//! the 753 keys written match a tag that way; the rest are the struct's
//! non-XML members (camera, editor and scenario state). That is what makes it
//! a direct oracle for every representation claim `sim::tuning::Slot` makes.

use std::cell::RefCell;
use std::fmt;

/// A growable array in **fixed-size chunks**, which is what keeps one
/// capture's parse from making the next one bigger.
///
/// A plain `Vec` doubles: the field arena of a 793 MB capture passes
/// through 300 MB, 600 MB and 1.2 GB blocks, and the two it abandons stay
/// abandoned — macOS's allocator caches a freed block of that size rather
/// than returning it, and a *differently* sized block from the next
/// capture cannot use it. The diff suite parses forty captures of forty
/// sizes one after another, so the process ratchets: before item 235 it
/// reached 15.3 GB serialized where its largest single test held 5.4 GB.
/// Chunks of one fixed size are the fix — every chunk any log frees fits
/// every chunk the next log wants, and nothing is ever copied on growth.
///
/// Item 260 briefly made each chunk an anonymous mapping, so that dropping
/// one returned it to the kernel rather than to an allocator that keeps
/// it; item 280 took that back with the rest of the `unsafe` (see
/// `crate::capture`). It costs nothing measurable, because 260's other
/// half — the lazy parse — took the arena of a 1.3 GB capture from 2,221
/// MiB to 17, and a ratchet over a chunk list that small is noise.
#[derive(Clone, Debug)]
struct Chunks<T: Copy + Default> {
    chunks: Vec<Box<[T]>>,
    len: usize,
}

/// 65,536 entries a chunk: 1 MiB of [`Field`], 1.8 MiB of [`Node`], 256 KiB
/// of child index. Small enough that the tail of the last chunk is not
/// worth counting, large enough that the chunk list is not.
const CHUNK: usize = 1 << 16;

impl<T: Copy + Default> Default for Chunks<T> {
    fn default() -> Self {
        Chunks {
            chunks: Vec::new(),
            len: 0,
        }
    }
}

impl<T: Copy + Default + PartialEq> PartialEq for Chunks<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && (0..self.len).all(|i| self.get(i) == other.get(i))
    }
}
impl<T: Copy + Default + PartialEq> Eq for Chunks<T> {}

impl<T: Copy + Default> Chunks<T> {
    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, i: usize) -> T {
        self.chunks[i / CHUNK][i % CHUNK]
    }

    fn set(&mut self, i: usize, v: T) {
        self.chunks[i / CHUNK][i % CHUNK] = v;
    }

    /// Forgets the entries without giving the chunks back — what a
    /// scratch arena reused frame after frame is for (item 260).
    fn clear(&mut self) {
        self.len = 0;
    }

    fn push(&mut self, v: T) {
        if self.len == self.chunks.len() * CHUNK {
            self.chunks
                .push(vec![T::default(); CHUNK].into_boxed_slice());
        }
        self.chunks[self.len / CHUNK][self.len % CHUNK] = v;
        self.len += 1;
    }

    fn extend(&mut self, vs: &[T]) {
        for &v in vs {
            self.push(v);
        }
    }
}

/// One field, as byte ranges into the log's own text.
///
/// **Why ranges and not `(&str, &str)`.** A 793 MB capture parses to 76.6 M
/// of these, and a pair of fat pointers is 32 bytes where a pair of
/// offsets is 16 — 2.3 GB against 1.2 GB, for text that can never exceed
/// the 4 GiB [`Log::parse`] asserts (the largest capture on disk is 1.4 GB).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Field {
    key_at: u32,
    key_len: u32,
    val_at: u32,
    val_len: u32,
}

/// One `BEGIN` block as it sits in the arena: its name and the ranges of
/// its fields and its children, all three into [`Log`]'s own flat vectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Node {
    name_at: u32,
    name_len: u32,
    indent: u32,
    fields_at: u32,
    fields_len: u32,
    kids_at: u32,
    kids_len: u32,
}

impl Node {
    /// `fields_len` for a block the first pass **indexed rather than
    /// read**: it has no fields and no children yet, so those four words
    /// carry the byte range of its own text instead — `fields_at` is where
    /// it starts and `kids_at` how long it is (item 260). Committing the
    /// block writes the real counts over both, which is what un-marks it.
    /// The alternative was two more words on every node, and item 235's
    /// guard is right that a node's width is worth 76.6 MB a byte.
    const LAZY: u32 = u32::MAX;

    /// The byte range of an indexed block's own text, if it still has one.
    fn body(&self) -> Option<(u32, u32)> {
        (self.fields_len == Node::LAZY).then_some((self.fields_at, self.kids_at))
    }
}

/// One `BEGIN` block: its name, its fields in file order, and its children.
///
/// A cursor into the [`Log`]'s arena rather than a node of its own, so it
/// is `Copy` and costs twelve bytes to pass around. Everything it hands
/// back — a name, a key, a value — is a slice of the original text.
#[derive(Clone, Copy)]
pub struct Block<'a> {
    log: &'a Log<'a>,
    node: u32,
}

impl fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("name", &self.name())
            .field("indent", &self.indent())
            .field("fields", &self.fields().len())
            .field("children", &self.children().len())
            .finish()
    }
}

/// Two blocks are the same block: the same arena, the same node.
impl PartialEq for Block<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.log, other.log) && self.node == other.node
    }
}
impl Eq for Block<'_> {}

impl<'a> Block<'a> {
    fn node(&self) -> Node {
        self.log.node(self.node)
    }

    /// Reads this block's own text into the arena if the first pass only
    /// indexed it. Everything that hands back a field or a child goes
    /// through here; a name and an indent come from the index and do not.
    fn ensure(&self) {
        if let Some(body) = self.node().body() {
            self.log.materialise(self.node, body);
        }
    }

    /// The key the read recorder files this block under: its arena and
    /// its node, which is what tells one `OBJECT` block from another.
    #[cfg(test)]
    pub(crate) fn read_key(&self) -> (usize, u32) {
        (self.log as *const Log<'_> as usize, self.node)
    }

    /// The text after `BEGIN `, e.g. `UNITDATA`, `FRAME 100`, `GAME INFO`.
    pub fn name(&self) -> &'a str {
        let n = self.node();
        self.log.slice(n.name_at, n.name_len)
    }

    /// Leading spaces on the `BEGIN` line.
    pub fn indent(&self) -> usize {
        self.node().indent as usize
    }

    /// `(key, value)` in file order. The value may be empty and keys repeat.
    ///
    /// Iterating the fields directly reads every key, and the read
    /// recorder ([`reads`]) notes it as `*`.
    pub fn fields(&self) -> Fields<'a> {
        #[cfg(test)]
        reads::note(self.log, self.node, "*");
        self.fields_raw()
    }

    fn fields_raw(&self) -> Fields<'a> {
        self.ensure();
        let n = self.node();
        Fields {
            log: self.log,
            at: n.fields_at as usize,
            end: (n.fields_at + n.fields_len) as usize,
        }
    }

    /// The child blocks, in file order.
    pub fn children(&self) -> Children<'a> {
        self.ensure();
        let n = self.node();
        Children {
            log: self.log,
            at: n.kids_at as usize,
            end: (n.kids_at + n.kids_len) as usize,
        }
    }

    /// The first value under `key`.
    pub fn get(&self, key: &str) -> Option<&'a str> {
        #[cfg(test)]
        reads::note(self.log, self.node, key);
        self.fields_raw().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    /// The first value under `key`, parsed as an integer.
    pub fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.trim().parse().ok()
    }

    /// Every value under `key`, in order — the array-constant shape.
    pub fn all(&self, key: &str) -> Vec<&'a str> {
        #[cfg(test)]
        reads::note(self.log, self.node, key);
        self.fields_raw()
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .collect()
    }

    /// The child blocks whose name is exactly `name`.
    pub fn kids<'n>(&self, name: &'n str) -> impl Iterator<Item = Block<'a>> + use<'a, 'n> {
        self.children().filter(move |b| b.name() == name)
    }

    /// The first child block named `name`.
    pub fn kid(&self, name: &str) -> Option<Block<'a>> {
        self.children().find(|b| b.name() == name)
    }

    /// The first child named `name`, searching depth-first through the
    /// whole subtree.
    pub fn find(&self, name: &str) -> Option<Block<'a>> {
        for c in self.children() {
            if c.name() == name {
                return Some(c);
            }
            if let Some(f) = c.find(name) {
                return Some(f);
            }
        }
        None
    }
}

/// **The read recorder** — which keys of which blocks a parse actually
/// asked for, so `crate::diff::coverage` can hold the dump's own field
/// list against the parser's (parked 488, the tenth pass). Ten instrument
/// defects in twenty-one landings were fields the original printed on
/// every frame and nothing read — `damage_frac` off the wrong record,
/// `build_masks` off the wrong block, a whole `AMMO` family for the life
/// of a capture — and each looked like agreement, because a field that is
/// not read cannot part. Test-only: the hooks in [`Block::get`],
/// [`Block::all`] and [`Block::fields`] compile to nothing outside
/// `cfg(test)`, and cost one thread-local look when the recorder is off.
///
/// A read is keyed on the arena and the node, never on a name: the same
/// `OBJECT` block sits under `UNITDATA` and under `WALLDATA`, and 484's
/// defect was exactly a key read on one of them and not the other. The
/// walk that turns nodes into paths is the guard's, after the parse.
#[cfg(test)]
pub(crate) mod reads {
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    /// `(arena address, node) → keys asked for`; `*` is a whole-fields
    /// iteration, which reads every key the block has.
    pub(crate) type Reads = BTreeMap<(usize, u32), BTreeSet<String>>;

    thread_local! {
        static ON: RefCell<Option<Reads>> = const { RefCell::new(None) };
    }

    /// Starts recording on this thread; a recording already open is
    /// dropped.
    pub(crate) fn start() {
        ON.with(|r| *r.borrow_mut() = Some(Reads::new()));
    }

    /// Stops recording and hands back everything noted since [`start`].
    pub(crate) fn stop() -> Reads {
        ON.with(|r| r.borrow_mut().take().unwrap_or_default())
    }

    pub(crate) fn note(log: &super::Log<'_>, node: u32, key: &str) {
        ON.with(|r| {
            if let Some(m) = r.borrow_mut().as_mut() {
                m.entry((log as *const super::Log<'_> as usize, node))
                    .or_default()
                    .insert(key.to_string());
            }
        });
    }
}

/// One block's fields, as a random-access range over the arena.
#[derive(Clone, Copy)]
pub struct Fields<'a> {
    log: &'a Log<'a>,
    at: usize,
    end: usize,
}

impl<'a> Fields<'a> {
    /// How many fields the block has.
    pub fn len(&self) -> usize {
        self.end - self.at
    }

    pub fn is_empty(&self) -> bool {
        self.at == self.end
    }

    /// The `i`th field.
    pub fn get(&self, i: usize) -> Option<(&'a str, &'a str)> {
        (self.at + i < self.end).then(|| self.log.field(self.at + i))
    }

    /// The fields from `i` on — the slicing a `[i..]` used to do.
    pub fn from(&self, i: usize) -> Fields<'a> {
        Fields {
            log: self.log,
            at: (self.at + i).min(self.end),
            end: self.end,
        }
    }

    /// The first `n`, for a caller that stops partway.
    pub fn head(&self, n: usize) -> Fields<'a> {
        Fields {
            log: self.log,
            at: self.at,
            end: (self.at + n).min(self.end),
        }
    }

    /// Materialised, for the readers that want a slice.
    pub fn to_vec(&self) -> Vec<(&'a str, &'a str)> {
        self.collect()
    }
}

impl<'a> Iterator for Fields<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        (self.at < self.end).then(|| {
            let f = self.log.field(self.at);
            self.at += 1;
            f
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Fields<'_> {}
impl DoubleEndedIterator for Fields<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.at < self.end).then(|| {
            self.end -= 1;
            self.log.field(self.end)
        })
    }
}

impl fmt::Debug for Fields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

/// One block's children, as a random-access range over the arena.
#[derive(Clone, Copy)]
pub struct Children<'a> {
    log: &'a Log<'a>,
    at: usize,
    end: usize,
}

impl<'a> Children<'a> {
    pub fn len(&self) -> usize {
        self.end - self.at
    }

    pub fn is_empty(&self) -> bool {
        self.at == self.end
    }

    pub fn get(&self, i: usize) -> Option<Block<'a>> {
        (self.at + i < self.end).then(|| Block {
            log: self.log,
            node: self.log.kid(self.at + i),
        })
    }

    /// The first `n` children — the slicing a `[..n]` used to do.
    pub fn head(&self, n: usize) -> Children<'a> {
        Children {
            log: self.log,
            at: self.at,
            end: (self.at + n).min(self.end),
        }
    }

    /// Every child but the first `n` — the `[n..]` half of [`Self::head`].
    pub fn tail(&self, n: usize) -> Children<'a> {
        Children {
            log: self.log,
            at: (self.at + n).min(self.end),
            end: self.end,
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = Block<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        (self.at < self.end).then(|| {
            let b = Block {
                log: self.log,
                node: self.log.kid(self.at),
            };
            self.at += 1;
            b
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Children<'_> {}

impl fmt::Debug for Children<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

/// A block being built: its scratch fields and children, which move into
/// the arena in one piece when it closes.
#[derive(Default)]
struct Open {
    indent: usize,
    node: u32,
    fields: Vec<Field>,
    kids: Vec<u32>,
}

/// A parsed log: the preamble fields and the top-level blocks.
///
/// **The tree is an arena.** Every block's fields and children live in
/// three flat vectors — `nodes`, `fields`, `kids` — with each block holding
/// index ranges into them. A tree of `Vec`s costs two allocations per
/// block, which on a 793 MB capture is 4.0 M live allocations and 1.3 GB of
/// doubling slack, and the slack is not the worst of it: freeing that many
/// small blocks leaves the allocator holding the pages, so a test suite
/// that parses one capture after another **ratchets** rather than returning
/// to its floor. Three big vectors are three `munmap`s (item 235).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Log<'a> {
    /// The text every slice in the arena points into.
    text: &'a str,
    /// Lines before the first `BEGIN`, as `(first token, rest)`.
    pub preamble: Vec<(&'a str, &'a str)>,
    /// Behind a cell because a block is read **when it is asked for**: a
    /// capture's frames are indexed by the first pass and parsed one at a
    /// time thereafter, so an accessor taking `&self` has to be able to
    /// grow the arena (item 260). Nothing borrows out of it — every read
    /// copies the fixed-size record out — so the cell is never held across
    /// a call.
    arena: RefCell<Arena>,
}

/// The three flat vectors every block's ranges point into, and the roots.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Arena {
    nodes: Chunks<Node>,
    fields: Chunks<Field>,
    kids: Chunks<u32>,
    roots: Vec<u32>,
}

impl<'a> Log<'a> {
    fn slice(&self, at: u32, len: u32) -> &'a str {
        &self.text[at as usize..(at + len) as usize]
    }

    fn field(&self, i: usize) -> (&'a str, &'a str) {
        let f = self.arena.borrow().fields.get(i);
        (
            self.slice(f.key_at, f.key_len),
            self.slice(f.val_at, f.val_len),
        )
    }

    fn node(&self, i: u32) -> Node {
        self.arena.borrow().nodes.get(i as usize)
    }

    fn kid(&self, i: usize) -> u32 {
        self.arena.borrow().kids.get(i)
    }

    /// The top-level blocks, in file order.
    pub fn roots(&'a self) -> Roots<'a> {
        Roots {
            log: self,
            at: 0,
            end: self.arena.borrow().roots.len(),
        }
    }

    /// [`Self::roots`] for a borrow that is **shorter** than the text's.
    ///
    /// `Log<'a>` is covariant in `'a`, so a scratch parse living on the
    /// stack — which is what a streaming scan of the frames is — can still
    /// be walked; `roots` cannot do it, because its `&'a self` pins the
    /// borrow to the text's own lifetime.
    fn roots_here(&self) -> Roots<'_> {
        Roots {
            log: self,
            at: 0,
            end: self.arena.borrow().roots.len(),
        }
    }

    /// Parses the whole text. Never fails: there is nothing in the grammar
    /// that can be malformed, only unexpected.
    ///
    /// # Panics
    ///
    /// On a text of 4 GiB or more, which no capture is and the arena's
    /// `u32` offsets could not address.
    pub fn parse(text: &'a str) -> Log<'a> {
        assert!(
            text.len() < u32::MAX as usize,
            "a gamelog of {} bytes is past the arena's 4 GiB reach",
            text.len()
        );
        let mut log = Log {
            text,
            ..Log::default()
        };
        fill(
            &mut log.arena.borrow_mut(),
            Some(&mut log.preamble),
            text,
            0,
            None,
            true,
        );
        log
    }

    /// [`Self::parse`] with the laziness off — the whole text read in one
    /// pass, which is what `the_lazy_tree_is_the_eager_tree` compares
    /// against. Also used for bounded indexed chunks that are immediately
    /// traversed in full; whole captures should keep the lazy entry point.
    pub(crate) fn parse_eager(text: &'a str) -> Log<'a> {
        assert!(
            text.len() < u32::MAX as usize,
            "a gamelog of {} bytes is past the arena's 4 GiB reach",
            text.len()
        );
        let mut log = Log {
            text,
            ..Log::default()
        };
        fill(
            &mut log.arena.borrow_mut(),
            Some(&mut log.preamble),
            text,
            0,
            None,
            false,
        );
        log
    }

    /// How many blocks the first pass indexed and has not yet read — what
    /// says a lazy/eager comparison is comparing anything at all.
    #[cfg(test)]
    pub(crate) fn indexed(&self) -> usize {
        let a = self.arena.borrow();
        (0..a.nodes.len())
            .filter(|&i| a.nodes.get(i).body().is_some())
            .count()
    }

    /// The root named `GAME`, as a node id, without pinning the borrow to
    /// the text's lifetime the way [`Self::game`] does.
    fn game_node(&self) -> Option<u32> {
        let a = self.arena.borrow();
        a.roots.iter().copied().find(|&id| {
            let n = a.nodes.get(id as usize);
            self.slice(n.name_at, n.name_len) == "GAME"
        })
    }

    /// Forgets everything read so far, keeping the chunks — a scratch
    /// arena between two frames.
    fn reset(&self) {
        let mut a = self.arena.borrow_mut();
        a.nodes.clear();
        a.fields.clear();
        a.kids.clear();
        a.roots.clear();
    }

    /// **Walks `GAME`'s children one at a time, on an arena thrown away
    /// between them** — the whole-file scan of item 260.
    ///
    /// A capture's frames are all children of `GAME`, and every whole-log
    /// product this module has (`frame_states`, `frame_seeds`,
    /// `anim_lengths`, the height grid, and the four walks `initial` used
    /// to do separately) wants each of them once and keeps something
    /// **owned**. Reading them into the log's own arena is what made a
    /// 1.3 GB capture cost 2.2 GB of nodes and fields; reading them into a
    /// scratch arena that is cleared before the next one costs one frame.
    ///
    /// A child the first pass read eagerly — everything before the first
    /// `FRAME` — is handed over from the log's own arena instead, so the
    /// walk is the complete list either way. `f` answers whether to carry
    /// on.
    fn scan_children(&self, mut f: impl FnMut(Block<'_>) -> bool) {
        let Some(game) = self.game_node() else {
            return;
        };
        let ids: Vec<u32> = {
            let a = self.arena.borrow();
            let n = a.nodes.get(game as usize);
            (0..n.kids_len)
                .map(|i| a.kids.get((n.kids_at + i) as usize))
                .collect()
        };
        let scratch = Log {
            text: self.text,
            ..Log::default()
        };
        for id in ids {
            match self.node(id).body() {
                Some((at, len)) => {
                    scratch.reset();
                    fill(
                        &mut scratch.arena.borrow_mut(),
                        None,
                        self.slice(at, len),
                        at,
                        None,
                        false,
                    );
                    let Some(b) = scratch.roots_here().next() else {
                        continue;
                    };
                    if !f(b) {
                        return;
                    }
                }
                None => {
                    let b = Block {
                        log: self,
                        node: id,
                    };
                    if !f(b) {
                        return;
                    }
                }
            }
        }
    }

    /// Reads an indexed block's own text into the arena — what
    /// [`Block::ensure`] calls the first time anyone asks a lazily indexed
    /// block for a field or a child.
    fn materialise(&self, id: u32, (at, len): (u32, u32)) {
        let region = self.slice(at, len);
        let mut arena = self.arena.borrow_mut();
        let roots_before = arena.roots.len();
        // The region begins on the block's own `BEGIN` line, so the pass
        // hands it back the node it already has rather than making one —
        // and committing it writes the real field and child counts over
        // the range that marked it lazy.
        fill(&mut arena, None, region, at, Some(id), false);
        // Its own block closed with nothing under it on the stack, so the
        // pass called it a root of the region; it is not one of the log's.
        arena.roots.truncate(roots_before);
        // A region always opens on the block's own `BEGIN`, so committing
        // it has written the counts over the marker. If it somehow has
        // not, clear the marker anyway: a block that stays lazy after
        // being read is one that is read again on every access, for ever.
        let mut n = arena.nodes.get(id as usize);
        if n.body().is_some() {
            n.fields_len = 0;
            n.kids_len = 0;
            arena.nodes.set(id as usize, n);
        }
    }
}

/// Reads `region` — a whole text, or one indexed block's own — into
/// `arena`.
///
/// `base` is `region`'s offset in the log's text, so that every span
/// recorded is an offset into *that* however deep the region sits.
/// `root` names a node already in the arena for the region's first block,
/// which is what makes a second pass over one block's text fill in the
/// node the first pass indexed rather than duplicate it. With `lazy`, the
/// pass stops reading at the first `FRAME` child of the top-level block
/// and **indexes** the rest; it answers whether it did.
fn fill<'a>(
    arena: &mut Arena,
    mut preamble: Option<&mut Vec<(&'a str, &'a str)>>,
    region: &'a str,
    base: u32,
    root: Option<u32>,
    lazy: bool,
) -> bool {
    {
        let log = &mut *arena;
        let base_ptr = region.as_ptr() as usize;
        // Every slice below is cut from `region`, so its offset is the
        // difference of the two pointers plus where the region begins.
        let off = |s: &str| base + (s.as_ptr() as usize - base_ptr) as u32;

        // The open-block stack, innermost last, indents strictly
        // increasing. A block leaves it when something at its own indent or
        // shallower arrives, and *then* it goes into the arena.
        let mut stack: Vec<Open> = Vec::new();
        // The block most recently closed by a field at its own indent: a
        // *run* of such fields (`leader_flags`, `leader_flags2`) all belong
        // to it, so it stays open — off the stack, but not yet committed —
        // until the run ends. Committing it is what attaches it to its
        // parent, so the parent must still be on the stack when it happens:
        // every path that could pop the parent flushes this first.
        let mut trailing: Option<Open> = None;
        // Closed blocks' scratch vectors, kept for the next block to use.
        let mut pool: Vec<Open> = Vec::new();
        // The node the region's first block is to be written back into,
        // when the region is one already-indexed block's own text.
        let mut first = root;

        // Whether the tail was indexed rather than read, and whether it is
        // still worth offering.
        let mut split = false;
        let mut try_lazy = lazy;
        let mut lines = region.lines();
        while let Some(line) = lines.next() {
            let trimmed = line.trim_start_matches(' ');
            if trimmed.is_empty() {
                continue;
            }
            let indent = line.len() - trimmed.len();
            let trimmed = trimmed.trim_end();
            if let Some(name) = trimmed.strip_prefix("BEGIN ") {
                if let Some(t) = trailing.take() {
                    commit(log, &mut stack, &mut pool, t);
                }
                while stack.last().is_some_and(|o| o.indent >= indent) {
                    let o = stack.pop().expect("just checked");
                    commit(log, &mut stack, &mut pool, o);
                }
                // **The lazy split** (item 260). A capture's frames are all
                // children of one block, and a test reads one of them; the
                // first `FRAME` child is therefore where reading stops and
                // indexing begins. The rest of the file is walked below
                // without a field being stored — each child of the open
                // block becomes a node holding nothing but its name and the
                // byte range of its own text, which `Block::ensure` reads
                // when someone asks it for something.
                if try_lazy
                    && stack.len() == 1
                    && indent > 0
                    && name.trim_start().starts_with("FRAME ")
                {
                    // The `BEGIN` line just read is the first span's, so
                    // the index picks up from it rather than after it.
                    let rest = &region[(off(line) - base) as usize..];
                    if let Some(stop) = index_tail(log, &mut stack, off, rest, indent) {
                        split = true;
                        // The index reads to the end unless a line closes
                        // the parent; the eager pass picks up there.
                        lines = region[(stop - base) as usize..].lines();
                        continue;
                    }
                    // The tail is a shape the index cannot model, so it is
                    // read eagerly — and never offered again. Asking at
                    // every later `FRAME` rescans the rest of the file once
                    // per frame, which is the file squared: a first draft
                    // did exactly that and got 34 tests done in the 220 s
                    // the whole suite used to take.
                    try_lazy = false;
                }
                let name = name.trim();
                let node = match first.take() {
                    // The region's own block already has a node: the pass
                    // that indexed it made one.
                    Some(id) => id,
                    None => {
                        let id = log.nodes.len() as u32;
                        log.nodes.push(Node {
                            name_at: off(name),
                            name_len: name.len() as u32,
                            indent: indent as u32,
                            ..Node::default()
                        });
                        id
                    }
                };
                let mut o = pool.pop().unwrap_or_default();
                o.indent = indent;
                o.node = node;
                stack.push(o);
            } else {
                let (key, value) = match trimmed.find(' ') {
                    Some(p) => (&trimmed[..p], trimmed[p + 1..].trim_start()),
                    // The empty value is cut from the text's own tail rather
                    // than written as `""`: every span in the arena is an
                    // offset into `text`, and a literal is not in it.
                    None => (trimmed, &trimmed[trimmed.len()..]),
                };
                let field = Field {
                    key_at: off(key),
                    key_len: key.len() as u32,
                    val_at: off(value),
                    val_len: value.len() as u32,
                };
                // Nothing writes `END` — `Log::end` emits only for a non-empty
                // name and the order writers all pass the empty string
                // (`docs/ORDERS.md` §11.1, second reading R7 L5) — so
                // indentation is the only thing that closes a block, and a
                // field one level deeper than a `BEGIN` belongs to it.
                //
                // **The writers are not consistent, and one shape is
                // genuinely ambiguous.** A field at *exactly* an open block's
                // own indent occurs in two forms that indentation cannot tell
                // apart:
                //
                //   BEGIN LEADERDATA   (1)      BEGIN TARGETORDER  (4)
                //    who 0             (2)       BEGIN UNITORDER   (5)
                //   leader_flags …     (1)        flags 0          (6)
                //                                ox 2001           (5)
                //
                // `leader_flags` is written *before* the `LEADERDATA` it
                // describes (so the block it just closed is the wrong home —
                // `records` re-zips those by position); `ox` belongs to
                // `TARGETORDER`, the *parent* of the `UNITORDER` it follows.
                // Both are one line at the open block's own indent.
                //
                // So the ambiguous field is recorded on **both** candidates:
                // the enclosing block the indent rule gives, and the block it
                // just closed. Every reader then finds it where it expects,
                // and the cost is one duplicated field on a block that will
                // not be asked for it. A field *shallower* than the open
                // block is not ambiguous and only closes.
                if trailing.as_ref().is_some_and(|t| t.indent != indent) {
                    let t = trailing.take().expect("just checked");
                    commit(log, &mut stack, &mut pool, t);
                }
                while stack.last().is_some_and(|o| o.indent >= indent) {
                    let o = stack.pop().expect("just checked");
                    if o.indent == indent && trailing.is_none() {
                        trailing = Some(o);
                    } else {
                        commit(log, &mut stack, &mut pool, o);
                    }
                }
                if let Some(t) = trailing.as_mut()
                    && t.indent == indent
                {
                    t.fields.push(field);
                }
                match stack.last_mut() {
                    Some(top) => top.fields.push(field),
                    None => {
                        if let Some(p) = preamble.as_deref_mut() {
                            p.push((key, value));
                        }
                    }
                }
            }
        }
        if let Some(t) = trailing.take() {
            commit(log, &mut stack, &mut pool, t);
        }
        while let Some(o) = stack.pop() {
            commit(log, &mut stack, &mut pool, o);
        }
        split
    }
}

/// Indexes the rest of a capture rather than reading it: every remaining
/// child of the open block becomes a node holding its name, its indent and
/// the byte range of its own text — and nothing else, so a 1.3 GB capture
/// whose frames nobody opens costs a few thousand nodes instead of
/// seventy-six million fields (item 260).
///
/// A span runs from its own `BEGIN` line to the next line at or above the
/// children's indent that opens one, so the trailing fields the writers put
/// at a block's *own* indent — `leader_flags` and its kind — fall inside the
/// span they belong to, exactly as the eager pass attributes them. The same
/// fields belong to the **parent** as well, and those are the only ones this
/// records.
///
/// Answers `false`, having changed nothing, if the tail does anything the
/// index cannot model — a line shallower than the children, or a block
/// opened after a field has already closed the span. Then the caller reads
/// the rest eagerly, and the only cost is the scan. Neither has been seen
/// in a capture; the check is what makes the laziness safe rather than
/// hopeful.
fn index_tail<'a>(
    log: &mut Arena,
    stack: &mut [Open],
    off: impl Fn(&str) -> u32,
    rest: &'a str,
    child_indent: usize,
) -> Option<u32> {
    let end_of_rest = off(rest) + rest.len() as u32;
    // `(name, indent, start)` per span, and the parent's own fields, both
    // held here until the whole tail has been read: a tail this cannot
    // model must leave the arena as it found it.
    let mut spans: Vec<(&'a str, u32)> = Vec::new();
    let mut fields: Vec<Field> = Vec::new();
    let mut ends: Vec<u32> = Vec::new();
    let mut span_open = false;
    // Where the index gave up and the eager pass takes over: a line
    // shallower than the children closes the parent, and every capture
    // ends with one (`GameInfo closing`, at indent 0, which belongs to
    // `GAME` *and* to the preamble). It is one line, so it is read rather
    // than modelled.
    let mut stop = end_of_rest;
    for line in rest.lines() {
        let trimmed = line.trim_start_matches(' ');
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let trimmed = trimmed.trim_end();
        let begins = trimmed.starts_with("BEGIN ");
        if indent < child_indent {
            stop = off(line);
            break;
        }
        if indent > child_indent {
            // Inside the current span — unless a field has already closed
            // it, in which case what opens here is a child of the *parent*
            // and the index has no node for it.
            if begins && !span_open {
                return None;
            }
            continue;
        }
        if begins {
            let name = trimmed["BEGIN ".len()..].trim();
            if !spans.is_empty() {
                ends.push(off(line));
            }
            spans.push((name, off(line)));
            span_open = true;
        } else {
            let (key, value) = match trimmed.find(' ') {
                Some(p) => (&trimmed[..p], trimmed[p + 1..].trim_start()),
                None => (trimmed, &trimmed[trimmed.len()..]),
            };
            fields.push(Field {
                key_at: off(key),
                key_len: key.len() as u32,
                val_at: off(value),
                val_len: value.len() as u32,
            });
            span_open = false;
        }
    }
    if spans.is_empty() {
        return None;
    }
    ends.push(stop);
    let parent = stack.last_mut()?;
    for ((name, start), end) in spans.into_iter().zip(ends) {
        let id = log.nodes.len() as u32;
        log.nodes.push(Node {
            name_at: off(name),
            name_len: name.len() as u32,
            indent: child_indent as u32,
            fields_at: start,
            fields_len: Node::LAZY,
            kids_at: end - start,
            kids_len: 0,
        });
        parent.kids.push(id);
    }
    parent.fields.extend(fields);
    Some(stop)
}

/// Moves a closed block's scratch into the arena, records its ranges, and
/// hands it to its parent — the block below it on the stack, or the roots.
fn commit(log: &mut Arena, stack: &mut [Open], pool: &mut Vec<Open>, mut o: Open) {
    let fields_at = log.fields.len() as u32;
    log.fields.extend(&o.fields);
    let kids_at = log.kids.len() as u32;
    log.kids.extend(&o.kids);
    let mut n = log.nodes.get(o.node as usize);
    n.fields_at = fields_at;
    n.fields_len = o.fields.len() as u32;
    n.kids_at = kids_at;
    n.kids_len = o.kids.len() as u32;
    log.nodes.set(o.node as usize, n);
    match stack.last_mut() {
        Some(parent) => parent.kids.push(o.node),
        None => log.roots.push(o.node),
    }
    o.fields.clear();
    o.kids.clear();
    // Two scratch vectors per open depth is all this ever needs, and the
    // pool is what keeps the block count from being an allocation count.
    if pool.len() < 64 {
        pool.push(o);
    }
}

/// The log's top-level blocks, as a range over the arena.
#[derive(Clone, Copy)]
pub struct Roots<'a> {
    log: &'a Log<'a>,
    at: usize,
    end: usize,
}

impl<'a> Roots<'a> {
    pub fn len(&self) -> usize {
        self.end - self.at
    }

    pub fn is_empty(&self) -> bool {
        self.at == self.end
    }
}

impl<'a> Iterator for Roots<'a> {
    type Item = Block<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        (self.at < self.end).then(|| {
            let b = Block {
                log: self.log,
                node: self.log.arena.borrow().roots[self.at],
            };
            self.at += 1;
            b
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Roots<'_> {}

impl<'a> Log<'a> {
    /// The first root named `name`.
    pub fn root(&'a self, name: &str) -> Option<Block<'a>> {
        self.roots().find(|b| b.name() == name)
    }

    /// The `BEGIN GAME` block, which holds the initial state and the frames.
    pub fn game(&'a self) -> Option<Block<'a>> {
        self.root("GAME")
    }

    /// The `BEGIN FRAME n` blocks in order, with their frame numbers.
    pub fn frames(&'a self) -> Vec<(i64, Block<'a>)> {
        let Some(game) = self.game() else {
            return Vec::new();
        };
        game.children()
            .filter_map(|b| {
                let n = b.name().strip_prefix("FRAME ")?.trim().parse().ok()?;
                Some((n, b))
            })
            .collect()
    }

    /// Every frame's `FULL DUMP` body, by frame number.
    ///
    /// A `DUMP_ALL` run writes the dump **twice** per frame — `full_dump`
    /// runs at `begin_frame` and at `end_frame` (`docs/SYNC.md` §1) — and
    /// the two carry the *same* state: the trailing one differs from the
    /// `FRAME` block's own by the checksum index, `turn_control`, the two
    /// command stamps and the timing counters, and by nothing else (run29,
    /// 2,583,636 lines compared line for line). So the second is a
    /// duplicate, and [`Self::frames`] is right to take the first.
    ///
    /// Where it is *not* a duplicate is the **end of a run**: `!quit`
    /// leaves a `FRAME n` block with no dump under it and the final
    /// `full_dump` lands after it, at `FRAME`'s own indent, which makes it
    /// a sibling the frame walk cannot reach. run29's free 15105 state was
    /// invisible for that reason alone. This walk takes a frame's nested
    /// dump when it has one and the sibling that follows it when it does
    /// not.
    pub fn dumps(&'a self) -> Vec<(i64, Block<'a>)> {
        let Some(game) = self.game() else {
            return Vec::new();
        };
        let mut out: Vec<(i64, Block<'a>)> = Vec::new();
        let mut open: Option<i64> = None;
        for b in game.children() {
            if let Some(n) = b
                .name()
                .strip_prefix("FRAME ")
                .and_then(|s| s.trim().parse().ok())
            {
                match b.kid("FULL DUMP") {
                    Some(d) => {
                        out.push((n, d));
                        open = None;
                    }
                    None => open = Some(n),
                }
            } else if b.name() == "FULL DUMP"
                && let Some(n) = open.take()
            {
                out.push((n, b));
            }
        }
        out
    }

    /// The setup path's checksum trace ([`Checksum`]), in call order. The
    /// records are written before `GameLog::begin_game`, at indent 0 with
    /// their `FILE`/`LINE` indented under nothing, so they land in the
    /// preamble as flat fields: `CHECKSUM n`, `FILE f`, `LINE l`, the
    /// subsystem checksums, then `game_random seed s`. A record without the
    /// seed line (`check_all_level < 14`) is dropped — it pins nothing.
    pub fn checksums(&'a self) -> Vec<Checksum<'a>> {
        checksums_in(self.preamble.iter().copied())
    }

    /// The sync stream's word at the **end of each frame**, from a
    /// `DUMP_ALL` dump whose `full_dump` runs at `begin_frame` and
    /// `end_frame` (`docs/SYNC.md` §1): `(engine frame, seed)`. The log's
    /// `FRAME n` block opens just before the `end_frame` dump of its frame
    /// — a `FULL DUMP` child whose first fields are the `say_checksum`
    /// record — so that record is the frame's last word: engine frame
    /// `n − 1`'s, the state frame `n` begins on. Empty for any other dump.
    pub fn frame_seeds(&'a self) -> Vec<(i64, u32)> {
        let mut out = Vec::new();
        self.scan_children(|b| {
            if let Some(n) = frame_number(b)
                && let Some(dump) = b.kid("FULL DUMP")
                && let Some(c) = checksums_in(dump.fields()).first()
            {
                out.push((n - 1, c.seed));
            }
            true
        });
        out
    }
}

/// Every `(gpiece, cur_anim, end_time)` under one block, the block itself
/// included — the walk behind [`Log::anim_lengths`], out here so that
/// [`Log::initial`] can fold it into its own single pass over the frames.
fn anims_in(b: Block<'_>, out: &mut Vec<(i64, i64, i64)>) {
    if b.name() == "GUY" {
        if let (Some(p), Some(a), Some(e)) = (b.int("gpiece"), b.int("cur_anim"), b.int("end_time"))
            && e > 0
        {
            out.push((p, a, e));
        }
        return;
    }
    for c in b.children() {
        anims_in(c, out);
    }
}

/// A block named `name`, taking the block itself as a candidate — which is
/// what a search over a *list* of blocks wants and [`Block::find`], which
/// only looks below, does not do.
fn find_here<'a>(b: Block<'a>, name: &str) -> Option<Block<'a>> {
    if b.name() == name {
        return Some(b);
    }
    b.find(name)
}

/// The number of a `FRAME n` block, or `None` for any other child.
fn frame_number(b: Block<'_>) -> Option<i64> {
    b.name().strip_prefix("FRAME ")?.trim().parse().ok()
}

/// The `CHECKSUM n / FILE / LINE / … / game_random seed` records among a
/// run of fields, in order.
fn checksums_in<'a>(fields: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<Checksum<'a>> {
    {
        let mut out = Vec::new();
        let mut cur: Option<Checksum<'a>> = None;
        for (key, value) in fields {
            match key {
                "CHECKSUM" => {
                    cur = value.trim().parse().ok().map(|n| Checksum {
                        n,
                        file: "",
                        line: 0,
                        seed: 0,
                    });
                }
                "FILE" => {
                    if let Some(c) = &mut cur {
                        c.file = value.trim();
                    }
                }
                "LINE" => {
                    if let Some(c) = &mut cur {
                        c.line = value.trim().parse().unwrap_or(0);
                    }
                }
                "game_random" => {
                    if let (Some(mut c), Some(s)) = (cur.take(), value.trim().strip_prefix("seed "))
                        && let Ok(seed) = s.trim().parse::<i64>()
                    {
                        c.seed = seed as u32;
                        out.push(c);
                    }
                }
                _ => {}
            }
        }
        out
    }
}

/// The `master_land_heights` carried by one block or any block under it.
///
/// `GameLog::dump_all@0092f2d0` prints `SimpleArray<float>::log_data` with
/// no block of its own, right after `UnbuiltForts::log_data`, so the
/// `length N` and the `list[scan]` values land on the `UnbuiltForts` block
/// at the same indent — the one whose `length` is over 1,000 (the forts
/// list itself is `length 0`). The waterline arrays that follow land there
/// too, so the values are taken in field order rather than by key.
fn heights_in(b: Block<'_>) -> Option<Vec<i64>> {
    if b.name() == "UnbuiltForts"
        && let Some(at) = b
            .fields()
            .position(|(k, v)| k == "length" && v.trim().parse::<usize>().is_ok_and(|n| n > 1000))
    {
        let n: usize = b
            .fields()
            .get(at)
            .map_or(0, |(_, v)| v.trim().parse().unwrap_or(0));
        let values: Vec<i64> = b
            .fields()
            .from(at + 1)
            .filter(|(k, _)| *k == "list[scan]")
            .take(n)
            .filter_map(|(_, v)| micro(v.trim()))
            .collect();
        if values.len() == n {
            return Some(values);
        }
    }
    b.children().find_map(heights_in)
}

impl<'a> Log<'a> {
    /// The `BUILDDATA` records of one frame's block — every building the
    /// original had standing at the end of sim-frame `n − 1`, with the
    /// `orig_type` and `build_masks` a `DUMP_ALL` block always carries.
    /// A capture that writes `BUILDS` per frame without `DUMP_ALL` — run80
    /// is one — nests nothing under a `FULL DUMP`, so the records are the
    /// `FRAME` block's own children and [`Log::dumps`] never sees them. The
    /// fallback only fires where this used to answer empty.
    pub fn frame_builds(&'a self, n: i64) -> Vec<BuildDump> {
        self.dumps()
            .into_iter()
            .find(|(f, _)| *f == n)
            .or_else(|| self.frames().into_iter().find(|(f, _)| *f == n))
            .map(|(_, b)| records(b, false).1)
            .unwrap_or_default()
    }

    /// The terrain's height grid from a `DUMP_ALL` dump, in millionths
    /// (see [`Initial::heights`]). `GameLog::dump_all@0092f2d0` prints
    /// `SimpleArray<float>::log_data(terrain->master_land_heights)` with no
    /// block of its own, right after `UnbuiltForts::log_data`, so the
    /// `length N` and the `list[scan]` values land on the `UnbuiltForts`
    /// block at the same indent — the one whose `length` is over 1,000
    /// (the forts list itself is `length 0`). Empty when no dump has it.
    pub fn terrain_heights(&'a self) -> Vec<i64> {
        for r in self.roots() {
            if Some(r.node) == self.game_node() {
                continue;
            }
            if let Some(h) = heights_in(r) {
                return h;
            }
        }
        let mut found = None;
        self.scan_children(|b| {
            found = heights_in(b);
            found.is_none()
        });
        found.unwrap_or_default()
    }

    /// The height grid **of one frame's block** — the same table, as it
    /// stood at the end of sim-frame `n − 1`. A whole-log
    /// [`Log::terrain_heights`] answers with the *first* it finds, which on
    /// a windowed capture is the start dump's; a building placed during the
    /// game re-terraforms the grid (`TerrainOut::terraform_for_building`,
    /// `docs/QUEUE.md` item 57), so anything comparing a mid-game search
    /// has to ask the frame rather than the game.
    pub fn frame_heights(&'a self, n: i64) -> Vec<i64> {
        self.frames()
            .into_iter()
            .find(|(f, _)| *f == n)
            .and_then(|(_, b)| heights_in(b))
            .unwrap_or_default()
    }

    /// One leader's whole `LEADERDATA` block at one frame — every field a
    /// `LEADERS=9` run prints (`docs/ORACLE.md`, "`LEADERS=9` is the census
    /// oracle"): the census counts by name, the `[scan]` arrays under their
    /// key (`Block::all("reg_land[scan]")`), and the `SITES`/`MAKELIST`
    /// children. The block's own `who` field identifies it; the
    /// `leader_flags` pair that precedes it in the file is not in it.
    pub fn leader_block(&'a self, frame: i64, who: i64) -> Option<Block<'a>> {
        let (_, b) = self.frames().into_iter().find(|(n, _)| *n == frame)?;
        // A `DUMP_ALL` frame nests its state under `FULL DUMP` (see
        // `records`); run20 is the first such capture whose leader record
        // is read this way.
        let b = b.kid("FULL DUMP").unwrap_or(b);
        b.kids("LEADERDATA").find(|l| l.int("who") == Some(who))
    }
}

/// A position in the engine's internal units, as the log writes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

/// One member of a unit, from a `GUY` block.
///
/// Below `DUMP_ALL` the per-frame `GUY` blocks are empty and the start dump
/// writes only `type`, the position and `angle`. A `DUMP_ALL` dump writes
/// `GuyData::log_data@005de6c0` whole — and the four fields after the
/// position are the animation clock (`docs/ANIM.md`): `cur_time`,
/// `end_time`, `cur_anim`, `gpiece`, with `last_time`, `guy_flags` and
/// `stopped` beside them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Guy {
    /// The unit type id. Written only in the start-of-game dump.
    pub kind: Option<i64>,
    pub pos: Option<Pos>,
    pub angle: Option<i64>,
    /// `GuyData::cur_time`: frames into the current animation.
    pub cur_time: Option<i64>,
    /// `GuyData::end_time`: the current animation's length in frames.
    pub end_time: Option<i64>,
    /// `GuyData::last_time`: `cur_time` before this frame's step, `−1`
    /// right after a `set_anim`.
    pub last_time: Option<i64>,
    /// `GuyData::cur_anim`: the `UnitAnim` index.
    pub cur_anim: Option<i64>,
    /// `GuyData::gpiece`: the graphic piece — the model whose animation
    /// packet the lengths come from.
    pub gpiece: Option<i64>,
    pub guy_flags: Option<i64>,
    pub stopped: Option<i64>,
    /// `GuyData::guy_num`: the member's index in its unit.
    pub guy_num: Option<i64>,
    /// `GuyData::hold_attack` (`+0x9e`) — **the attack this figure owes
    /// because it was still walking or still turning when the swing asked
    /// for it**, which `Guy::move` pays on a later frame
    /// (`docs/ANIM.md` §6.2).
    ///
    /// Parsed by item 510, which is the frame it decides: an attack that
    /// lands here costs the wrap **one** draw, and one that lands in
    /// [`Guy::queued_attack`] costs it **two**.
    pub hold_attack: Option<i64>,
    /// `GuyData::queued_attack` (`+0xa0`) — the attack asked for while an
    /// attack was **already playing**, which `Guy::inc_time` pays inside
    /// its own wrap loop at the shared `+0x271`. Same encoding: `1` when
    /// the request carried its third argument, the slot itself otherwise.
    pub queued_attack: Option<i64>,
    /// `GuyData::ox` / `whom` (`+0x8e` / `+0x9f`) — **what the figure last
    /// swung at**, `−1` until its first strike. `Unit::set_attack` writes
    /// it on every strike, and `Unit::fight`'s recharging arm reads guy
    /// 0's against the order's target (item 530, `docs/ORDERS.md` §22).
    pub ox: Option<i64>,
    pub whom: Option<i64>,
    /// `GuyData::des_x` / `des_y` (`+0x5c` / `+0x60`) — **where this
    /// figure is told to be**, which for a tracked crew figure is its
    /// leader's point rotated by its track offset and rewritten several
    /// times a frame (`docs/MOVEMENT.md`, "Who writes it, and when").
    /// `Guy::set_anim`'s walking-guy early return is `des != pos`, so
    /// this is the field that decides whether an idle request on a
    /// walking figure costs a draw.
    pub des: Option<Pos>,
    /// `GuyData::des_angle` (`+0x64`) — the angle the same writers hand
    /// it, and what `Guy::move`'s arrival arm tests the facing against.
    pub des_angle: Option<i64>,
    /// `GuyData::track_dx` / `track_dy` (`+0x54` / `+0x58`), the art's
    /// ground-track offset: non-zero is exactly what gives a crew figure
    /// a body of its own (`docs/MOVEMENT.md`, "Where the track offset
    /// comes from").
    pub track: Option<(i64, i64)>,
    /// `GuyData::last_x` / `last_y` (`+0x68` / `+0x6c`) — where the figure
    /// was before this frame's step, which `Guy::move` writes from `x`/`y`
    /// at its very first statement.
    ///
    /// **The one place it is not the previous position is a write**:
    /// `Guy::set_new_location(…, 1)` sets it from the point it is putting
    /// the figure on, so `last_pos == pos` on a block is the signature of
    /// a figure that was *placed* rather than one that walked. That is how
    /// an age is read off a dump — `Leader::gain_tech`'s `is_age_type` arm
    /// places every one of the leader's units where it already stands
    /// (`docs/TECH.md`, "An age snaps every figure").
    pub last_pos: Option<Pos>,
    /// `GuyData::last_speed` (`+0x80`) — the distance this figure moved
    /// on its last step, zeroed at the head of `Guy::move`'s standing
    /// arm.
    pub last_speed: Option<i64>,
    /// `GuyData::avg_speed` (`+0x84`) — `(avg · 3 + last_speed) / 4`,
    /// rewritten at the foot of every `Guy::move`. It is the **walk
    /// slot's own input**: `Guy::set_anim`'s walk arm slogs below six
    /// tenths of the type's base and jogs above eleven, so this pair
    /// decides whether an arriving figure stands on `CHAR_WALK` — and
    /// the arrival draw tests that slot (`docs/ANIM.md` §4, §4.3).
    pub avg_speed: Option<i64>,
}

impl Guy {
    /// Whether this record carries the clock — a `DUMP_ALL` block.
    pub fn has_clock(&self) -> bool {
        self.cur_time.is_some() && self.end_time.is_some() && self.cur_anim.is_some()
    }
}

/// One entry of a unit's `OrderList`, as `UNITS=3` writes it
/// (`docs/ORDERS.md` §11.1).
///
/// The list is logged **newest first**, so the last block in a unit's dump is
/// the order being executed. The `type`/`metric` lines sit on the enclosing
/// `UNITDATA` block rather than inside the order's own block, so they are
/// paired with the order bodies by position.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrderDump {
    /// `type` — the `OrderIndex` value (`sim::orders::index`).
    pub index: i64,
    /// The list node's `metric` byte: written 0, logged, never read.
    pub metric: i64,
    /// The block's own name, e.g. `GATHERORDER`, `EXPLORETOORDER`.
    pub kind: String,
    /// `UnitOrder::flags` — bit `4` is the action bit.
    pub flags: i64,
    /// `TargetOrder`'s three, for the kinds that have them.
    pub ox: Option<i64>,
    pub whom: Option<i64>,
    pub uid: Option<i64>,
    /// `GATHERORDER`, the whole row (`docs/ORDERS.md` §6.4): the chosen
    /// resource tile, the camp/tile phase, the countdown, the distance
    /// weight the tile choice scores with, and whether the worker has been
    /// out yet. The first pass carried three of these and compared none of
    /// them, and the tile it did not carry is what pinned player 1's score
    /// at frame 4 for three days.
    pub tx: Option<i64>,
    pub ty: Option<i64>,
    pub build_type: Option<i64>,
    pub been_there: Option<i64>,
    pub goto_build: Option<i64>,
    pub non_flat_gather: Option<i64>,
    pub dist_mod: Option<i64>,
    pub wait: Option<i64>,
    /// `MOVEORDER` and its subclasses.
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub dest: Option<i64>,
    /// The rest of the `MOVEORDER` row, in the order the block writes it
    /// (`docs/ORDERS.md` §11.1's table): the whole record, so a widening
    /// has something to compare rather than the three fields the first
    /// pass happened to need.
    pub angle: Option<i64>,
    pub tolerance: Option<i64>,
    pub pause: Option<i64>,
    pub retry: Option<i64>,
    pub attempts: Option<i64>,
    pub timer: Option<i64>,
    pub facing: Option<i64>,
    pub dest_x: Option<i64>,
    pub dest_y: Option<i64>,
    pub last_x: Option<i64>,
    pub last_y: Option<i64>,
    pub coll_x: Option<i64>,
    pub coll_y: Option<i64>,
    pub orig_x: Option<i64>,
    pub orig_y: Option<i64>,
    /// `MoveOrder::off_x`/`off_y` — the destination's offset **inside its
    /// world cell**, `x mod 0x300` (§11.2's table), not a formation slot.
    pub off_x: Option<i64>,
    pub off_y: Option<i64>,
    /// `GroupOrder::log_data@00485640`'s row, on the `GROUPORDER` base of a
    /// `GroupMoveOrder` — the order `Group::action_move_near` gives a member
    /// (`docs/GROUPS.md` §6.6 step 6), first seen in run31.
    ///
    /// `oxx` is the **leader's object**, so the record names outright what
    /// `GroupData::find_leader` chose; `form_id` is the member's own index
    /// into the group's parallel arrays, which is what pairs an order with a
    /// `GROUPDATA` slot.
    pub oxx: Option<i64>,
    pub whose: Option<i64>,
    pub group_angle: Option<i64>,
    pub group_id: Option<i64>,
    pub form_id: Option<i64>,
    /// `GroupMoveOrder`'s own last field, past both bases.
    pub in_group: Option<i64>,
    /// `ATTACKORDER`'s own fields, past the `TARGETORDER` base.
    pub mandatory: Option<i64>,
    pub defensive: Option<i64>,
    pub in_range: Option<i64>,
    pub ever_in_range: Option<i64>,
    pub new_ord: Option<i64>,
    pub def_x: Option<i64>,
    pub def_y: Option<i64>,
}

impl OrderDump {
    /// The `OrderIndex` this record's **block name** implies
    /// (`docs/ORDERS.md` §1.2's table), or `None` for a name the table
    /// does not list.
    ///
    /// The name and the `type` line are two independent statements of the
    /// same quantity: `type` is written by `OrderList::log_data` on the
    /// enclosing `UNITDATA` and paired **positionally**, the name by the
    /// order's own `log_data`. So they disagree exactly when the pairing
    /// has slid — which is what a block name the walk drops does, and has
    /// done once already (`GroupMoveOrder`, the one lower-case name in
    /// the family). Nothing compared them until item 237.
    pub fn named_index(&self) -> Option<i64> {
        Some(match self.kind.to_ascii_uppercase().as_str() {
            "MOVEORDER" => 1,
            "ATTACKTOORDER" => 2,
            "EXPLORETOORDER" => 3,
            "FLEETOORDER" => 4,
            "BUILDORDER" => 6,
            "GATHERORDER" => 7,
            "BOARDORDER" => 8,
            "AWAITBOARDORDER" => 9,
            "ATTACKORDER" => 10,
            "FOLLOWORDER" => 11,
            "GUARDORDER" => 12,
            "REPAIRORDER" => 13,
            "CASTORDER" => 14,
            "TRADEORDER" => 15,
            "STRAFEORDER" => 16,
            "AIRPATROLORDER" => 17,
            "FORMORDER" => 18,
            "GROUPMOVEORDER" => 19,
            "GROUPATTACKORDER" => 20,
            "GROUPATTACKTOORDER" => 21,
            "GROUPPATROLORDER" => 22,
            "ATTACKGROUNDORDER" => 23,
            "AIRATTACKGROUNDORDER" => 24,
            "SPECIALANIMORDER" => 25,
            "GARRISONORDER" => 26,
            "THINKORDER" => 27,
            _ => return None,
        })
    }

    /// The action bit (`UnitOrder::flags & 4`, §1.3): this order is an intent
    /// rather than a transit leg.
    pub const fn is_action(&self) -> bool {
        self.flags & 4 != 0
    }
}

/// One entry of a unit's path stack — `Stack<PathData>::log_data` (§11.1).
///
/// The stack is written **bottom first**, and the bottom is the goal:
/// `find_path` pushes the destination with the final flag and then the
/// waypoints on top of it, so the *last* block logged is the waypoint the
/// unit is walking to now. `sim::orders::PathData` is held in a `Vec` used
/// the same way — pushed and popped at the end — so the two are in the same
/// order without reversing either.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PathDump {
    pub to: (i64, i64),
    pub tolerance: i64,
    /// Bit `1` is the final segment (the goal).
    pub flags: i64,
}

/// One unit: the `Object` base plus its members.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitDump {
    pub flags: i64,
    /// The object number within its owner's unit array.
    pub o: i64,
    pub who: i64,
    pub pos: Pos,
    /// `UnitData::angle` (`+0x50`), the unit's heading — the `UNITDATA`
    /// level's own `angle` line, not a guy's. It is the angle
    /// `Group::update_positions@00713810` rotates the slot table by when
    /// this unit executes a group move (`docs/GROUPS.md` §6.6).
    pub angle: Option<i64>,
    pub guys: Vec<Guy>,
    /// The order list, **newest first** as the log writes it — empty below
    /// `UNITS=3`. [`UnitDump::current_order`] is the one being executed.
    pub orders: Vec<OrderDump>,
    /// The path stack, bottom (the goal) first — the `STACK<TYPE>` block that
    /// precedes the order list, empty below `UNITS=3` and for a unit that is
    /// not walking a path.
    pub path: Vec<PathDump>,
    /// The order layer's own `UNITDATA` fields — the ones `docs/ORDERS.md`
    /// and `docs/GROUPS.md` cite by name, so a scene built from a block can
    /// stand a group up as the original had it.
    ///
    /// `UnitData::group` (`+0x80`): the group slot this unit belongs to, −1
    /// none. It is the back-pointer `Group::normalize` culls on.
    pub group: Option<i64>,
    /// `UnitData +0xaa` / `+0xab`: the formation index and width the last
    /// group move wrote (`docs/GROUPS.md` §4.4, §6.6 step 1). `get_form`
    /// and `get_form_mod_option` read them back off the members, which is
    /// why a halted group can carry `form −1` while every member still
    /// carries the formation it was last laid out in.
    pub form: Option<i64>,
    pub form_mod: Option<i64>,
    /// `UnitData::stance` (§8): the worker or combat stance option.
    pub stance: Option<i64>,
    /// `UnitData::orders_x`/`orders_y`: the final destination of the
    /// leading run of transit moves (`docs/ORDERS.md` §2).
    pub orders_x: Option<i64>,
    pub orders_y: Option<i64>,
    /// `UnitData::dest_angle` and `UnitData::tolerance`.
    pub dest_angle: Option<i64>,
    pub tolerance: Option<i64>,
    /// `UnitData::path_recursion` and `UnitData::idle` (§2.4, §4.5).
    pub path_recursion: Option<i64>,
    pub idle: Option<i64>,
    /// `UnitData::unit_masks` / `unit_masks2` — the bit fields §6.6's
    /// predicates read.
    pub unit_masks: Option<i64>,
    pub unit_masks2: Option<i64>,
    /// `UnitData::attrition` (`+158`), the pending tick period in frames
    /// that `Unit::process_attrition` writes on each 32-frame refresh
    /// (`docs/ATTRITION.md`, "The cadence"). Printed on every `UNITDATA`
    /// since the first capture and parsed from item 552, when chapter four
    /// put a squad on hostile ground.
    pub attrition: Option<i64>,
    /// `UnitData::myspeed`, the output of `get_speed`'s pipeline
    /// (`docs/MOVEMENT.md`).
    pub myspeed: Option<i64>,
    /// `UnitData::o_up`: the captain this unit reports to, −1 when it is
    /// one itself — `is_captain`, which `find_leader` and `get_num_cap`
    /// both open on.
    pub o_up: Option<i64>,
    /// `UnitData::o_down`: the next figure down the squad chain, −1 at its
    /// tail. A dead figure's slot stays in the chain, re-appended at the
    /// tail by `Unit::close` (`docs/COMBAT.md` §47.2).
    pub o_down: Option<i64>,
    /// `UnitData::inside_up`: the building this unit is garrisoned in.
    pub inside_up: Option<i64>,
    /// **The hit-point record**, all three of it, written inside the
    /// `OBJECT` block at every detail level — `ObjectData::myhits`, the
    /// **squad's** whole hit points (`docs/COMBAT.md` §7.3: `update_hits`
    /// writes one number onto every figure and `take_damage` divides it on
    /// the way in); `ObjectData::damage`, the whole points this **figure**
    /// has taken; and `ObjectData::damage_frac`, the sixteenths under them
    /// (§7.2 step 4).
    ///
    /// `damage_frac` was parsed on `BUILDDATA` from item 394 and not here
    /// until item 484, which is the pair one record over: a field the dump
    /// prints on every block of every capture, and the one that decides
    /// *which frame* the next whole point lands on.
    pub myhits: Option<i64>,
    pub damage: Option<i64>,
    pub damage_frac: Option<i64>,
    /// `ObjectData::mylos` — the line of sight `Unit::update_los` last
    /// computed for this object, in tiles (`docs/VISION.md` §2). On the
    /// `OBJECT` level, beside `myhits`.
    pub mylos: Option<i64>,
    /// `ObjectData::infiltrated` and `ObjectData::visible` — the two masks
    /// `Object::update_seen` reveals *with* rather than *for*. Zero in
    /// every capture on disk; kept so the record is compared whole.
    pub infiltrated: Option<i64>,
    pub visible: Option<i64>,
    /// `ObjectData::hold_frames` (`+0x32`) — how long a slot is held past
    /// the death of the figure in it (`docs/COMBAT.md` §11, §42.3),
    /// printed on the `OBJECT` level at every detail and read by nothing
    /// until item 491.
    ///
    /// Its three writers are all on a **dead** object (`Object::die`,
    /// `Ammo::inc_time`'s bump of the *shooter*, `DeathObj::inc_time`), so
    /// what the comparison asserts is that it is **zero on every living
    /// unit-frame** — and that is the check §9.2's backwards sentence
    /// never had. `coverage`'s pin had carried it since the guard was
    /// built.
    pub hold_frames: Option<i64>,
    /// The collision block, written at every detail level
    /// (`docs/COLLISION.md`): the counter, the frame of the last one, and
    /// what was in the way.
    pub collide: Option<i64>,
    pub collide_frame: Option<i64>,
    pub collide_o: Option<i64>,
    pub collide_who: Option<i64>,
    pub collide_guy: Option<i64>,
    /// `UnitData::recharging` (`+0xae`) — the reload clock, and the gate
    /// `Unit::fight@005fd4d0:102` returns on before it ever asks whether
    /// the target is still a target. The whole record is compared on the
    /// AI headline's own block and this field was not in it until item
    /// 463, which is how a squad that keeps a dead target's order for the
    /// length of its reload read as a pathing divergence.
    pub recharging: Option<i64>,
    /// **The overkill window** (`docs/COMBAT.md` §7.1 step 2, §41) —
    /// `UnitData::damage_frame`, the sim frame of the first hit inside the
    /// current window, and `damage_o`/`damage_who`, the captain and owner
    /// of whoever struck it. All three are written at `UNITDATA`'s own
    /// indent on every unit of every block of every capture; nothing
    /// parsed them until item 485, which is why item 484's wound ladder
    /// had to read the frame of each hit off the accumulator that moved
    /// rather than off the dump's own stamp of it.
    ///
    /// A never-hit unit carries `damage_frame 0`, `damage_o -1`,
    /// `damage_who 0`.
    pub damage_frame: Option<i64>,
    pub damage_o: Option<i64>,
    pub damage_who: Option<i64>,
    /// `UnitData::safe` — the cooldown a failed 48-grid search buys.
    pub safe: Option<i64>,
    /// **`UnitData::start_dist` (`+0x130`) — the one dumped witness that a
    /// 48-grid search suspended.** `PathFinder::astar_path@00683770`'s
    /// suspend block is its **only** writer in the whole executable
    /// (grepped, item 301), so a non-zero value says the unit stopped a
    /// search short of its goal and handed it the containers, and the value
    /// is that search's start-to-goal Manhattan
    /// (`docs/PATHFINDER.md` §4.3 step 3, §18). Nothing clears it: like
    /// `collide_frame` it is a permanent stamp, so a **change** dates a
    /// suspend and a value does not.
    pub start_dist: Option<i64>,
    /// `ObjectData::uid` — the **identity** behind the per-player `o`,
    /// written on the `OBJECT` level beside the cell chain. `o` is a slot
    /// and is handed back out: a unit that dies frees its number, and the
    /// next unit that player trains is born into it with every field of
    /// the record reset. `uid` is what tells the two apart, which is what
    /// a claim about a *permanent* stamp needs — run16's `1/9` reads
    /// `start_dist` 768 under `uid 17` and 0 under `uid 25`
    /// (`docs/PATHFINDER.md` §18.6).
    pub uid: Option<i64>,
    /// `ObjectData::down`/`down_who` and `up`/`up_who`: this object's place
    /// in its world cell's chain, on the `OBJECT` level
    /// (`docs/COLLISION.md` §3).
    pub down: Option<i64>,
    pub down_who: Option<i64>,
    pub up: Option<i64>,
    pub up_who: Option<i64>,
}

impl UnitDump {
    /// The order the unit is executing: the **last** block logged, because
    /// `OrderList::log_data` walks the ring from the tail (§11.1).
    pub fn current_order(&self) -> Option<&OrderDump> {
        self.orders.last()
    }

    /// The order list front-first — current order first, the way
    /// `sim::Sim`'s `VecDeque` holds it. The log writes it the other way.
    pub fn orders_front_first(&self) -> impl Iterator<Item = &OrderDump> {
        self.orders.iter().rev()
    }
}

/// One building: the `Object` base as written under `BUILDDATA`/`WALLDATA`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildDump {
    pub flags: i64,
    pub o: i64,
    pub who: i64,
    pub pos: Pos,
    /// **The damage pair**, one block in from `BUILDDATA` under `OBJECT`,
    /// and unparsed until item 394 — `damage` is the whole hit points
    /// lost and `damage_frac` the sixteenths under them
    /// (`docs/COMBAT.md` §7.2 step 4).
    ///
    /// The pair is what dates a landing to a frame: Great Lakes' farm
    /// `0/2004` goes `0/0` → `1/10` in one block because **two** arrows
    /// land on sim-frame 9451, thirteen sixteenths apiece, and the first
    /// of them moves only the fraction (§20).
    ///
    /// The rest of the `ObjectData` half — `healing`, `hold_frames`,
    /// `infiltrated`, `visible`, `launch_frames`, `inside_down`,
    /// `inside_down_who`, `near_o`, `near_who` — is written on every
    /// `BUILDDATA` record and is still unparsed, deliberately: this
    /// crate's `Building` models none of them, and `ledger.rs`'s rule is
    /// that a parsed field is a compared field. That is the next
    /// widening, and it is a modelling item rather than a parsing one.
    pub damage: Option<i64>,
    pub damage_frac: Option<i64>,
    /// `orig_type` — the `TypeIndex` the building was created as. Written at
    /// **`BUILDS=6`** and above, and the only type the dump ever carries.
    pub orig_type: Option<i64>,
    /// `max_age` — the age byte `Leader::gain_tech` step 6 rewrites on
    /// **every** building of the player the moment an age is gained, which
    /// makes it the one field of the dump that dates an age to a block
    /// (`docs/TECH.md`, "An age snaps every figure").
    pub max_age: Option<i64>,
    /// `WallData::build_masks`, whose `0x100` is "my roads want replanning"
    /// — the schedule's own field (`docs/ROADS.md` §1), written from
    /// **`BUILDS=1`**. `None` where the level did not print it.
    pub build_masks: Option<i64>,
    /// `BuildData::gather_from`, the `MiningList` — a woodcutter's or mine's
    /// resource tiles, in tiles, in the order the original keeps them.
    /// **`BUILDS=7`**, and written *flat*: `MiningList::log_data` opens no
    /// `BEGIN` of its own, so after `mtn`/`cliff` come the array's
    /// `length size increment flags` and then one `tx`/`ty` pair per entry,
    /// all at `BUILDDATA`'s own field indent. Nothing else at that level
    /// writes `tx`, so the pairs are unambiguous.
    pub gather_from: Vec<(i64, i64)>,
    /// The `MiningList`'s own header, as `ArrayBase<TCoordData>::log_data`
    /// writes it just before the pairs: `length` is the live entry count and
    /// `size` the allocation. The length is carried separately from
    /// [`BuildDump::gather_from`]`.len()` on purpose — they are the same
    /// number on a well-formed record, and a capture where they are not is a
    /// parse that has drifted rather than a game that has.
    pub mining_len: Option<i64>,
    pub mining_size: Option<i64>,
    /// `MiningList::mtn` and `::cliff` — which mountain range or cliff the
    /// list was taken from, `−1` on a timber list. `find_gather_tcoords`
    /// writes one of them the first time it fills a metal building's list,
    /// and reads them back to know the list has been filled before.
    pub mtn: Option<i64>,
    pub cliff: Option<i64>,
    /// `BuildData::gather_down` — the head of the chain of units registered
    /// as gathering here, by object number, `−1` for none
    /// (`docs/ORDERS.md` §6.1). Written at **`BUILDS=1`**.
    pub gather_down: Option<i64>,
    /// **The construction clock**, `WallData`'s own three — the block one
    /// level in from `BUILDDATA`, beside `gpiece` and `frame_started`.
    /// `job_counter` climbs in hundredths of a frame towards
    /// `constr_time`, which `Wall::update_construct_time@0063d560` bakes
    /// from the type's `job_time × 100` and the owner's nation, wonder,
    /// rare and tech modifiers (`docs/CITIES.md` §3.2). Written from
    /// **`BUILDS=1`**, parsed and compared nowhere until item 261 — where
    /// the original's Tower read **90909** against this crate's 100000
    /// for four hundred frames.
    pub job_counter: Option<i64>,
    pub constr_time: Option<i64>,
    /// `WallData::construct_hits` — the hit points the site has been
    /// raised to, logged as `(int)construct_hits`.
    pub construct_hits: Option<i64>,
    /// `WallData::ever_seen` (`Wall +0x62`) and `ever_seen_completed`
    /// (`+0x63`) — one bit per player: who has ever had this building's
    /// footprint in **current** line of sight, and who has had it there
    /// while it was finished. `Wall::check_ever_seen@0063ce70` grows them
    /// every eighth frame, and a newly arrived foreign bit is what makes
    /// the building light its own footprint into that player's fog
    /// (`docs/VISION.md` §6.1). They sit on the `WALLDATA` block. Written
    /// from **`BUILDS=1`**; unparsed until item 322, which is how Great
    /// Lakes' 8031 stood for a day on a reveal nobody could name.
    pub ever_seen: Option<i64>,
    pub ever_seen_completed: Option<i64>,
    /// `BuildData::queued` (`+0x82`) — how many entries of the queue are
    /// live. Written from **`BUILDS=1`**; `None` below it.
    pub queued: Option<i64>,
    /// The production queue, capacity slots and all —
    /// `BuildQueue::log_data`'s `queue[scan]` run under `BEGIN BUILDQUEUE`,
    /// one nine-line group per slot, `queue_size` of them. Only the first
    /// [`BuildDump::queued`] are live; the rest hold whatever the array was
    /// last left with, which is `type −1` on a queue that has never been
    /// used and `type 0` on the tail of one that has.
    pub queue: Vec<QueueItemDump>,
}

/// One `QueueItem` as the dump writes it (`docs/PRODUCTION.md`, "The queue
/// record"): the counter, the type, and the three `(good, cost)` pairs the
/// entry remembers of the six a price can have.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueueItemDump {
    /// `type` — a `TypeIndex`, over units and techs alike.
    pub ty: i64,
    /// `job_counter`, in hundredths of a frame.
    pub job_counter: i64,
    pub cost: [i64; 3],
    pub good: [i64; 3],
}

/// One leader's level-0 fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LeaderDump {
    pub who: i64,
    pub tribe: i64,
    pub defeated_by: i64,
    pub gov: i64,
    pub score: i64,
    pub leader_flags: i64,
    pub leader_flags2: i64,
    /// `LeaderData::diplos` (`+0x74`, `int[8]`) — 0 war, 1 peace, 2
    /// alliance, and the diagonal is 2 (`docs/ARMY.md` §1). Every capture
    /// so far opens with the whole off-diagonal at **0**, which is what a
    /// Quick Battle is; the harness read none of it until `crate::danger`
    /// needed `do_danger`'s three arms told apart.
    pub diplos: Vec<i64>,
}

/// One slot of a leader's make list — the `MAKEOBJECT` block
/// `MakeObject::log_data@006d8aa0` prints under a `LEADERS=9` `LEADERDATA`
/// record, all ten fields in the struct's own order (`docs/AI.md` §2.6).
/// Eleven per leader; `t −1` is an empty slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MakeObjectDump {
    pub t: i64,
    pub val: i64,
    pub escrow: i64,
    pub city: i64,
    pub up: i64,
    pub o: i64,
    pub num: i64,
    pub cat: i64,
    pub wx: i64,
    pub wy: i64,
}

impl<'a> Block<'a> {
    /// The `MAKEOBJECT` children of a `LEADERDATA` block, in slot order —
    /// eleven at `LEADERS=9`, none below it. A field the block lacks reads
    /// as zero, so a caller that needs the record whole checks the count.
    pub fn make_list(&self) -> Vec<MakeObjectDump> {
        self.kids("MAKEOBJECT")
            .map(|m| {
                let i = |k| m.int(k).unwrap_or(0);
                MakeObjectDump {
                    t: i("t"),
                    val: i("val"),
                    escrow: i("escrow"),
                    city: i("city"),
                    up: i("up"),
                    o: i("o"),
                    num: i("num"),
                    cat: i("cat"),
                    wx: i("wx"),
                    wy: i("wy"),
                }
            })
            .collect()
    }
}

/// One `CITY` record, whole — every field `CityData::log_data@004895c0`
/// writes, in the order it writes them.
///
/// The record is unconditional past its own gate: a slot is printed only
/// when `city_flags & 1` is set, and then **every** field below is written,
/// with no detail level and no per-field test. So a missing key here is a
/// hand-written sample, not a capture — which is why each parses
/// `unwrap_or(0)` rather than carrying an `Option`.
///
/// The two strings are not carried. `name` and `id` are written bare, with
/// no key, and `id` is written only when it is non-empty; a bare line is
/// not a field the block parser can address, and the name is drawn from
/// `City::generate_name`'s table rather than being sim state this crate
/// holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CityDump {
    /// `(this->x).value` / `(this->y).value` — the city point in internal
    /// units, the same scale a `SUBOBJECT`'s `x_internal` is written in.
    pub x: i64,
    pub y: i64,
    pub pop: i64,
    pub who: i64,
    /// The nation the city is assimilated to; `-1` on a slot never founded.
    pub race: i64,
    /// `docs/CITIES.md` §1.4 — the bit table.
    pub city_flags: i64,
    /// The slot number **within the owner's own city array**, which is what
    /// an `ARMYDATA`'s `city` indexes (`docs/ARMY.md` §13).
    pub city: i64,
    pub attack_stamp: i64,
    pub raid_stamp: i64,
    pub reduce_stamp: i64,
    pub capture_stamp: i64,
    pub assimilation_timer: i64,
    pub capture_strength: i64,
    /// `Array<CaravanLink>::log_data(&this->vans)`: the array header —
    /// `length`, `size`, `increment`, `flags` — and then one nested block
    /// per link, each carrying `cara` and `who`. Only the links are
    /// carried; the header's capacity is the container's, not the city's.
    pub vans: Vec<(i64, i64)>,
    /// The city building's object number, and its region.
    pub o: i64,
    pub reg: i64,
    pub scouted: i64,
    pub in_port: i64,
    pub peasant_dist: i64,
    pub trade_val: i64,
    pub free: i64,
    pub busy: i64,
    pub gatherers: i64,
    pub ocean: i64,
    pub land: i64,
    pub filled: i64,
    pub bordering: i64,
    pub ocean_filled: i64,
    pub dock_tile: i64,
    pub was_capital_flags: i64,
    pub space: [i64; 3],
    pub ter: [i64; 6],
}

/// One `CITY` block, typed.
///
/// The four container fields between `capture_strength` and `o` —
/// `length`, `size`, `increment`, `flags` — belong to the **caravan
/// array**, not to the city, which is why nothing here reads them; the
/// enclosing `CITIES` array writes its own three a level up, and a `Block`
/// keeps only the first value under a key anyway.
pub(crate) fn city_of(b: Block<'_>) -> CityDump {
    let i = |k: &str| b.int(k).unwrap_or(0);
    let arr = |k: &str| -> Vec<i64> {
        b.all(k)
            .iter()
            .map(|v| v.trim().parse().unwrap_or(0))
            .collect()
    };
    let at = |v: &[i64], n: usize| v.get(n).copied().unwrap_or(0);
    let space = arr("space[scan]");
    let ter = arr("ter[scan]");
    CityDump {
        x: i("x"),
        y: i("y"),
        pop: i("pop"),
        who: i("who"),
        race: b.int("race").unwrap_or(-1),
        city_flags: i("city_flags"),
        city: i("city"),
        attack_stamp: i("attack_stamp"),
        raid_stamp: i("raid_stamp"),
        reduce_stamp: i("reduce_stamp"),
        capture_stamp: i("capture_stamp"),
        assimilation_timer: i("assimilation_timer"),
        capture_strength: i("capture_strength"),
        vans: b
            .children()
            .filter(|c| c.get("cara").is_some())
            .map(|c| (c.int("cara").unwrap_or(-1), c.int("who").unwrap_or(-1)))
            .collect(),
        o: b.int("o").unwrap_or(-1),
        reg: b.int("reg").unwrap_or(-1),
        scouted: i("scouted"),
        in_port: i("in_port"),
        peasant_dist: i("peasant_dist"),
        trade_val: i("trade_val"),
        free: i("free"),
        busy: i("busy"),
        gatherers: i("gatherers"),
        ocean: i("ocean"),
        land: i("land"),
        filled: i("filled"),
        bordering: i("bordering"),
        ocean_filled: i("ocean_filled"),
        dock_tile: i("dock_tile"),
        was_capital_flags: i("was_capital_flags"),
        space: [at(&space, 0), at(&space, 1), at(&space, 2)],
        ter: [
            at(&ter, 0),
            at(&ter, 1),
            at(&ter, 2),
            at(&ter, 3),
            at(&ter, 4),
            at(&ter, 5),
        ],
    }
}

/// One member's row of a `GROUPDATA` record — the six parallel arrays
/// `GroupData::log_data` writes per member, in the order it writes them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupMemberDump {
    /// `list[i]`, the member object's number.
    pub o: i64,
    /// `off_x[i]`/`off_y[i]` — `Form::compute_dests`' slot offset,
    /// **before** the leader's heading is applied.
    pub off_x: i64,
    pub off_y: i64,
    /// `curr_x[i]`/`curr_y[i]` — the same offset after
    /// `Group::update_positions` rotates it (`docs/GROUPS.md` §6.6).
    pub curr_x: i64,
    pub curr_y: i64,
    /// `angles[i]`, the per-slot facing byte a move order packs into its
    /// top byte. Written as a **signed** char.
    pub angle: i64,
}

/// One `GROUPDATA` record, whole — the twenty scalars
/// `GroupData::log_data@0045e1d0` writes in the order it writes them, and
/// the six parallel per-member arrays after them (`docs/GROUPS.md` §1).
///
/// **`march` (`+0x4b`) is the only field of the struct the engine never
/// logs**, which is why there is no field for it here.
///
/// The pool is dumped as 512 of these — 8 leaders × 64 slots, in `id`
/// order — directly under `FULL DUMP`, and the hotkey groups separately
/// under `HOTKEYGROUPS`, where each `HOTKEYGROUPDATA` wrapper **nests** its
/// own `GROUPDATA`. [`groups`] returns the pool only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupDump {
    pub id: i64,
    pub who: i64,
    pub num: i64,
    pub army: i64,
    pub ox: i64,
    pub oy: i64,
    pub o_dist: i64,
    pub o_angle: i64,
    pub buildings: i64,
    pub disband: i64,
    pub order_num: i64,
    pub priority: i64,
    pub stamp: i64,
    pub role: i64,
    pub form: i64,
    pub think_frame: i64,
    pub facing: i64,
    pub new_speed: i64,
    pub speed: i64,
    pub form_num: i64,
    pub members: Vec<GroupMemberDump>,
}

/// The twenty scalars in the order `GroupData::log_data` writes them. The
/// order is an assertion, not a convenience: a dump whose prefix differs
/// means the writer changed.
pub const GROUP_FIELDS: [&str; 20] = [
    "id",
    "who",
    "num",
    "army",
    "ox",
    "oy",
    "o_dist",
    "o_angle",
    "buildings",
    "disband",
    "order_num",
    "priority",
    "stamp",
    "role",
    "form",
    "think_frame",
    "facing",
    "new_speed",
    "speed",
    "form_num",
];

/// One cell of the `WORLD` block at `WORLD ≥ 3` — the `WData` record
/// `WData::log_data@006af7e0` prints (`docs/ORACLE.md`, "The map is a dump
/// too"). Absent fields (a lower threshold) stay at their defaults.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellDump<'a> {
    /// The level-2 line: the terrain kind's `land_key[]` name.
    pub land: &'a str,
    pub flags: i64,
    pub goods: i64,
    /// The level-4 line: the flag words, or the "none" string.
    pub flag_words: &'a str,
    pub who: i64,
    pub who2: i64,
    pub region: i64,
    pub region2: i64,
    pub val: i64,
    pub land_sub: i64,
    pub light: i64,
    pub blocked: i64,
    pub bad: i64,
    pub solid: i64,
    pub down: i64,
    pub down_who: i64,
    pub was_seen: i64,
}

/// The cells of a `WORLD` block's field run, in the writer's order
/// (`wdata[0..size]`, row-major). The record has no `BEGIN` of its own —
/// `WData::log_data` writes flat lines — so a cell starts at each `flags`
/// key, the bare line before it is the terrain name, and the bare line
/// between `goods` and `who` is the flag words. The block's own scalars
/// (`xs`, `seed`, …) precede the first cell and are skipped; the per-tile
/// and per-fog runs after the last cell are not read here.
pub fn world_cells<'a>(fields: &[(&'a str, &'a str)]) -> Vec<CellDump<'a>> {
    let mut cells: Vec<CellDump<'a>> = Vec::new();
    let mut pending_land: &'a str = "";
    let mut after_goods = false;
    let int = |v: &str| v.trim().parse::<i64>().unwrap_or(0);
    for &(k, v) in fields {
        // A cell's `flags` follows its bare land line; the `flags` of the
        // `SimpleArray` blocks after the cells follow `increment -1`.
        if k == "flags" && !pending_land.is_empty() {
            cells.push(CellDump {
                land: pending_land,
                flags: int(v),
                ..CellDump::default()
            });
            pending_land = "";
            after_goods = false;
            continue;
        }
        let Some(cell) = cells.last_mut() else {
            // The block's scalars, before the first cell.
            if v.trim().is_empty() {
                pending_land = k;
            }
            continue;
        };
        if v.trim().is_empty() {
            if after_goods && cell.who == 0 && cell.flag_words.is_empty() && cell.region == 0 {
                cell.flag_words = k;
            } else {
                pending_land = k;
            }
            continue;
        }
        match k {
            "goods" => {
                cell.goods = int(v);
                after_goods = true;
            }
            // The level-4 line is `NO FEATURE` on a plain cell — two tokens,
            // so it parses as a key with a value.
            "NO" if after_goods && cell.flag_words.is_empty() => cell.flag_words = k,
            "who" => cell.who = int(v),
            "who2" => cell.who2 = int(v),
            "region" => cell.region = int(v),
            "region2" => cell.region2 = int(v),
            "val" => cell.val = int(v),
            "land_sub" => cell.land_sub = int(v),
            "light" => cell.light = int(v),
            "blocked" => cell.blocked = int(v),
            "bad" => cell.bad = int(v),
            "solid" => cell.solid = int(v),
            "down" => cell.down = int(v),
            "down_who" => cell.down_who = int(v),
            "was_seen" => cell.was_seen = int(v),
            _ => {}
        }
    }
    cells
}

/// The per-tile masks of a `WORLD` block — the `tdata[scan].mask` run after
/// the cells (`TData.mask`, one `ushort` a tile, `tile_xs × tile_ys` of
/// them row-major; `docs/CITIES.md` §2.3 for the bits). Empty below
/// `WORLD=5`.
pub fn world_tiles(fields: &[(&str, &str)]) -> Vec<u16> {
    fields
        .iter()
        .filter(|(k, _)| *k == "tdata[scan].mask")
        .map(|(_, v)| v.trim().parse::<u32>().unwrap_or(0) as u16)
        .collect()
}

/// The fog grid's `seen2` bytes — `WorldData +0x160`, `fog_xs × fog_ys`
/// (two per cell each way), one bit per player — which
/// `WorldData::was_seen@006b53f0` answers from (`seen2[fy × fog_xs + fx] &
/// ally_mask`). The WORLD block prints them after the tile masks as
/// `seen[scan]`/`seen2[scan]`/`seen3[scan]` triplets, one per fog cell
/// (run20, 2026-08-25: 14,400 each on a 60×60 map, 357 with each player's
/// bit at the start of the game — the initial vision).
pub fn world_fog(fields: &[(&str, &str)]) -> Vec<u8> {
    fields
        .iter()
        .filter(|(k, _)| *k == "seen2[scan]")
        .map(|(_, v)| v.trim().parse::<u32>().unwrap_or(0) as u8)
        .collect()
}

/// The danger map — `WorldData::danger[8]` (`world+0x13c`), printed by
/// `WorldData::log_data@006b6080:628` as `danger[who][scan]`, eight rows of
/// `reg_size` `int`s back to back in leader order.
///
/// `reg_size` is `reg_xs × reg_ys` and the block does not say what it is, so
/// the caller divides by eight: the whole run is `8 × reg_size` long, and a
/// 60×60 map's is 7,200. It is the one AI-visible grid a `WORLD` dump
/// carries, and nothing compared it until `crate::danger` had a writer.
pub fn world_danger(fields: &[(&str, &str)]) -> Vec<i64> {
    fields
        .iter()
        .filter(|(k, _)| *k == "danger[who][scan]")
        .filter_map(|(_, v)| v.trim().parse::<i64>().ok())
        .collect()
}

/// A named constant from the `CONSTANTS` block: scalar or array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantDump<'a> {
    /// The key as written, with any `[scan]`/`[scan2]` suffix removed.
    pub name: &'a str,
    /// The values in order; one for a scalar, several for an array.
    pub values: Vec<i64>,
    /// Whether the key carried an array suffix.
    pub array: bool,
}

/// One `GameLog::say_checksum@00930b30` record — the setup path's own sync
/// trace (`docs/ORACLE.md`, "The setup path's checksum trace is the RNG
/// state"). Every call site in `Game::init`, `init_rules_and_teams`,
/// `Setup::build_game`, `Map::make`, `Terrain::init` and `Leader::init`
/// prints `CHECKSUM n`, the source `FILE` and `LINE`, one checksum per
/// walked subsystem and, at `check_all_level ≥ 14` in `rise.ini`,
/// **`game_random seed`** — the sync stream's state at that point. `seed`
/// is the raw 32-bit word (the log prints it signed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checksum<'a> {
    pub n: i64,
    pub file: &'a str,
    pub line: i64,
    pub seed: u32,
}

/// A `%f`-printed number as exact millionths: `369.375000` → `369375000`,
/// `-2.5` → `-2500000`. Decimals beyond the sixth are dropped; anything
/// that is not a decimal number is `None`.
pub fn micro(s: &str) -> Option<i64> {
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    let whole: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let mut f: i64 = 0;
    for (i, c) in frac.chars().take(6).enumerate() {
        let d = c.to_digit(10)? as i64;
        f += d * 10i64.pow(5 - i as u32);
    }
    if frac.chars().any(|c| !c.is_ascii_digit()) {
        return None;
    }
    let v = whole * 1_000_000 + f;
    Some(if neg { -v } else { v })
}

/// The state written once, before frame 1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Initial<'a> {
    /// `GAME INFO` → `GAMEINFO` fields, e.g. `MAP_SIZE`, `(int)seed`.
    pub game_info: Vec<(&'a str, &'a str)>,
    pub players: Vec<Vec<(&'a str, &'a str)>>,
    pub world: Vec<(&'a str, &'a str)>,
    pub cities: Vec<CityDump>,
    pub constants: Vec<ConstantDump<'a>>,
    pub units: Vec<UnitDump>,
    pub builds: Vec<BuildDump>,
    pub leaders: Vec<LeaderDump>,
    /// The setup path's checksum trace, in call order — empty unless the
    /// run had `[Misc Logging] CHECKSUM ≥ 1` and `check_all_level ≥ 14`.
    pub checksums: Vec<Checksum<'a>>,
    /// The terrain's `master_land_heights` — `(4·xs + 1) × (4·ys + 1)`
    /// corner heights, row-major, each in **millionths** — the log prints
    /// the floats with six decimals, which is finer than an `f32`'s
    /// spacing at these magnitudes, so the text names the float exactly;
    /// but the floats are **not** on a ⅛ grid (run12 has 39,746 of 58,081
    /// off it, because `terraform_for_building` has averaged every
    /// footprint by the time the dump is written), so a consumer that
    /// wants the original's arithmetic must do it in `f32`, not in
    /// millionths (`docs/ROADS.md` §7, `docs/QUEUE.md`). Only a `DUMP_ALL`
    /// dump carries it (`docs/ORACLE.md`); empty otherwise.
    pub heights: Vec<i64>,
    /// The `HERDS` block's `HERD` records — only a `DUMP_ALL` dump prints
    /// them (`docs/SYNC.md` §3.2); empty otherwise.
    pub herds: Vec<HerdDump>,
    /// The `FULL DUMP` block's `GOOD` records, in the original's own
    /// `goods` order — the fish, whales, oil and rares the map generator
    /// laid down. Only an initial `FULL DUMP` prints them; empty
    /// otherwise, and a world stood up without them has no goods, so
    /// `find_good_at` answers nothing everywhere.
    pub goods: Vec<GoodDump<'a>>,
    /// The `REGIONS` block's `REGION` records, in the original's own region
    /// order. Only an `InitialDump` prints them; empty otherwise, and a
    /// world stood up without them has every region's `flags` at 0.
    pub regions: Vec<RegionDump>,
    /// `Farms::log_data`'s list, in the order `Farms::inc_time` walks it —
    /// the order the sprout draw is spent in (`docs/SYNC.md` §4.1). Only a
    /// `DUMP_ALL` dump prints it; empty otherwise.
    pub farms: Vec<FarmDump>,
    /// The sync stream's word at the end of each engine frame, from the
    /// per-frame `say_checksum` records of a `DUMP_ALL` dump
    /// ([`Log::frame_seeds`]); empty otherwise.
    pub frame_seeds: Vec<(i64, u32)>,
    /// Every `(gpiece, cur_anim) → end_time` a `DUMP_ALL` dump's `GUY`
    /// blocks show, over every state it printed ([`Log::anim_lengths`]) —
    /// the animation lengths, which are art data the sim takes as an input
    /// (`docs/ANIM.md`). Empty for any other dump.
    pub anim_lengths: Vec<(i64, i64, i64)>,
    /// Per engine frame a `DUMP_ALL` dump traced, every unit's `(who, o,
    /// guys)` at the end of that frame — the clocks the harness installs
    /// beside the frame's word. Empty for any other dump.
    pub frame_guys: FrameGuys,
    /// Per engine frame, every unit whose `GUY` blocks carry a **position**
    /// — the figures' own `x`, `y` and `angle`, which a dump writes at a
    /// lower detail than the clock and which [`Initial::frame_guys`]
    /// therefore filters out.
    ///
    /// This is the oracle for the body: guy 0's is the unit's own, and a
    /// crew guy's is a second body walking its own destination
    /// (`docs/MOVEMENT.md`, "The follower's destination"). Nothing is
    /// installed from it — it is compared.
    pub frame_bodies: FrameGuys,
    /// **The one field no dump fills.** Each pasture's five animals as
    /// `Farms::add_animals` created them, borrowed from the run's own
    /// *trace* ([`crate::trace::Trace::add_animals`]) because owner 9 is in
    /// no dump block at all and the draws are spent inside
    /// `Setup::build_empire` (`docs/SYNC.md` §3.11). One `Vec` a pasture,
    /// in creation order; empty leaves the simulation's stand-in.
    pub pasture: Vec<Vec<sim::farms::AnimalSeed>>,
}

/// Every unit's clocks at the end of each traced engine frame.
pub type FrameGuys = Vec<(i64, Vec<FrameUnit>)>;

/// One unit as a traced frame's dump left it: who it is, its figures'
/// clocks, and whether its order list was **empty** at that moment.
///
/// The last is what separates a unit the original had mid-walk from one that
/// has just arrived — an arrival keeps the walk animation for the frame that
/// notices it, and the difference decides whether the clock is a correction
/// or a corruption ([`crate::diff::Built::tick`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameUnit {
    pub who: i64,
    pub o: i64,
    pub guys: Vec<Guy>,
    /// `orders.is_empty()` — only meaningful at `UNITS=3`, where the list is
    /// written at all.
    pub orderless: bool,
    /// The unit's position, and the goal of a leading `MOVEORDER` if it has
    /// one — what [`crate::diff::Built::tick`] re-seats gaia's animals from.
    pub pos: Pos,
    pub goal: Option<Pos>,
}

/// One `REGION` record of the `REGIONS` block — `Regions::log_data@
/// 00681280` walks every slot, so the list is index-aligned with the
/// original's own region numbering and includes the empty ones.
///
/// Only the three fields a consumer has needed so far. `flags` is the one
/// that matters: **bit `8` is the resource-region flag**, which is
/// `Region::go_here@006810f0`'s whole first arm and so the gate on the AI
/// ever sending anyone to another island (`docs/TRANSPORT.md` §7, §9.4).
/// Nothing in the world's own cells carries it — it is the map generator's
/// (`Map::region_flags`), and this record is where it can be read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegionDump {
    pub region: i64,
    pub flags: i64,
    pub size: i64,
}

/// One `GOOD` record of a `FULL DUMP`: the good type's **name** (the
/// record's one valueless field, `Type::get_name`), its object number and
/// its position in the original's own units.
///
/// The list is in `goods` order, which is what `WData.down_who` indexes
/// when `down` is `−2` (`Objects::init_good@00653f30`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoodDump<'a> {
    pub name: &'a str,
    pub o: i64,
    pub x: i64,
    pub y: i64,
    /// `SubObjectData::flags` — bit 0 is "live".
    pub flags: i64,
}

/// One `HERD` record: the home cell, the wander centre, the animal type
/// and the flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HerdDump {
    pub cx: i64,
    pub cy: i64,
    pub wx: i64,
    pub wy: i64,
    pub t: i64,
    pub herd_flags: i64,
}

/// One `FarmStruct` of `Farms`' list: the building it belongs to, whether
/// the slot is live, and the crop — `farm_type == 1` is the pasture, which
/// grows nothing and keeps five animals of owner 9 (`docs/SYNC.md` §3.6).
///
/// The list has no block of its own: `Farms::log_data` writes `who`, `o`,
/// the sixteen cells, the twenty-five corner heights and then `valid` and
/// `farm_type` as **flat fields of the enclosing dump**, so a record is
/// read as "the `who`/`o` pair that most recently preceded a `farm_type`".
/// The sixteen cells are printed **in memory order** — `FarmStruct` holds
/// `float[4][4] percent` at `+0x8` and `uchar[4][4] status` at `+0xac`, and
/// `Farms::log_data` walks them as one flat run of `percent`/`status`
/// pairs. The farmer's cell is `status[dx][dy]` (`docs/ORDERS.md` §6.5), so
/// the index here is `dx * 4 + dy`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FarmDump {
    pub who: i64,
    pub o: i64,
    pub valid: i64,
    pub farm_type: i64,
    /// `status[dx][dy]` — 0 empty, 1 growing, 2 ripe, 3 cut.
    pub status: Vec<i64>,
    /// `percent[dx][dy]`, as the count of `0.005f` adds the simulation
    /// keeps ([`sim::farms::Farm::adds`]): the printed float divided by
    /// `0.005` and rounded. Six decimals over a `0.005` step leaves no
    /// ambiguity, and [`farms_of`] refuses a value that is not within a
    /// tenth of an add of an integer.
    pub adds: Vec<i64>,
}

/// One frame's worth of state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub n: i64,
    pub units: Vec<UnitDump>,
    pub builds: Vec<BuildDump>,
    pub leaders: Vec<LeaderDump>,
    /// The frame's `CITIES` list, whole. `CityData::log_data` is called
    /// from `Cities::log_data` at every detail level a frame block is
    /// written at, so this is populated on every capture that dumps
    /// anything per frame — which nothing compared until item 154.
    pub cities: Vec<CityDump>,
    /// The frame's `DEATH_OBJS` list — one record per death object still
    /// playing its animation (`docs/COMBAT.md` §42.1).
    ///
    /// A whole record family nobody had opened: item 485 read a
    /// `first_frame` out of the raw text by hand, and `coverage`'s pin
    /// carried all six of its keys as unread until item 491.
    pub deaths: Vec<DeathDump>,
}

/// One `DEATH_OBJS` record — `DeathObjData::log_data@008d55e0`, written at
/// `DEATHS=1` and above (`docs/COMBAT.md` §42.1).
///
/// The six keys detail 1 prints, in the order the logger writes them.
/// `cur_anim` is the whole of the record for this crate: it is
/// `dtype * 2 + 0xd + roll % 2`, so it carries both the death class
/// `do_damage` decided and the parity of the draw `Unit::close+0xcb6`
/// spent — which is how `dtype 2` stopped being a reading.
///
/// `gpiece` is the unit's own death piece (`DeathObj::init`'s
/// `vtable[0x178]`), which this crate loads no art for; it is parsed and
/// not compared (§42.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeathDump {
    pub valid: Option<i64>,
    pub first_frame: Option<i64>,
    pub cur_anim: Option<i64>,
    pub who: Option<i64>,
    pub o: Option<i64>,
    pub gpiece: Option<i64>,
}

/// One `DEATH_OBJS` block's six keys.
pub(crate) fn death_of(d: Block<'_>) -> DeathDump {
    DeathDump {
        valid: d.int("valid"),
        first_frame: d.int("first_frame"),
        cur_anim: d.int("cur_anim"),
        who: d.int("who"),
        o: d.int("o"),
        gpiece: d.int("gpiece"),
    }
}

/// Every `DEATH_OBJS` record of a frame block, in the order the original
/// walks its list.
pub(crate) fn deaths_of(b: Block<'_>) -> Vec<DeathDump> {
    b.kids("DEATH_OBJS").map(death_of).collect()
}

fn pos_of(b: Block<'_>) -> Pos {
    Pos {
        x: b.int("x_internal").unwrap_or(0),
        y: b.int("y_internal").unwrap_or(0),
        z: b.int("z_internal").unwrap_or(0),
    }
}

/// The `OBJECT` → `SUBOBJECT` base of a unit or building block.
fn object_base(b: Block<'_>) -> Option<(i64, i64, i64, Pos)> {
    let sub = b.find("SUBOBJECT")?;
    Some((
        sub.int("flags")?,
        sub.int("o")?,
        sub.int("who")?,
        pos_of(sub),
    ))
}

/// The order list of a `UNITDATA` block (§11.1).
///
/// `OrderList::log_data` writes, on the *enclosing* block, one `type` and one
/// `metric` line per order, each followed by the order's own `BEGIN <KIND>`
/// child. `Block` keeps fields and children in separate vectors, so the
/// pairing is by position: the k-th `type` belongs to the k-th `*ORDER`
/// child. Nothing else writes a bare `type` at this level — a `GUY`'s is
/// inside its own block.
fn orders_of(b: Block<'_>) -> Vec<OrderDump> {
    let ints = |k: &str| -> Vec<i64> {
        b.all(k)
            .iter()
            .filter_map(|v| v.trim().parse::<i64>().ok())
            .collect()
    };
    let types = ints("type");
    let metrics = ints("metric");
    // Every order block's name ends in "order" — but **not always in caps**:
    // `GroupMoveOrder::log_data@00485910` opens `"GroupMoveOrder"` where its
    // two bases open `"MOVEORDER"` and `"GROUPORDER"`. A case-sensitive test
    // dropped it silently, which also slid every later `type`/`metric` onto
    // the wrong body, since the pairing is positional (§11.1).
    b.children()
        .filter(|c| c.name().to_ascii_uppercase().ends_with("ORDER"))
        .enumerate()
        .map(|(i, o)| {
            // `ox/whom/uid` live on the `TARGETORDER` sub-block and `flags` on
            // `UNITORDER`, both reached depth-first; the kind's own fields are
            // on the outer block. A field shallower than the open block's
            // indent closes it, which is what puts `ox` on `TARGETORDER`
            // rather than on `UNITORDER` (§11.1's third trap).
            let target = o.find("TARGETORDER");
            let unit_order = o.find("UNITORDER");
            // The class chain is the block nesting, exactly as
            // `BUILDDATA` → `OBJECT` → `SUBOBJECT` is: an
            // `ATTACKTOORDER` writes its `MOVEORDER` base as a child, and
            // a plain `MOVEORDER` *is* the block. `find` looks at
            // children only, so the block itself has to be offered first.
            let base = |name: &str| {
                if o.name() == name {
                    Some(o)
                } else {
                    o.find(name)
                }
            };
            let mv = base("MOVEORDER");
            let atk = base("ATTACKORDER");
            let grp = base("GROUPORDER");
            let mv_int = |k: &str| mv.and_then(|m| m.int(k));
            let atk_int = |k: &str| atk.and_then(|a| a.int(k));
            let grp_int = |k: &str| grp.and_then(|g| g.int(k));
            OrderDump {
                index: types.get(i).copied().unwrap_or(-1),
                metric: metrics.get(i).copied().unwrap_or(0),
                kind: o.name().to_string(),
                flags: unit_order.and_then(|u| u.int("flags")).unwrap_or(0),
                ox: target.and_then(|t| t.int("ox")),
                whom: target.and_then(|t| t.int("whom")),
                uid: target.and_then(|t| t.int("uid")),
                tx: o.int("tx"),
                ty: o.int("ty"),
                build_type: o.int("build_type"),
                been_there: o.int("been_there"),
                goto_build: o.int("goto_build"),
                non_flat_gather: o.int("non_flat_gather"),
                dist_mod: o.int("dist_mod"),
                wait: o.int("wait"),
                x: o.find("MOVEORDER")
                    .map_or_else(|| o.int("x"), |m| m.int("x")),
                y: o.find("MOVEORDER")
                    .map_or_else(|| o.int("y"), |m| m.int("y")),
                dest: o
                    .find("MOVEORDER")
                    .map_or_else(|| o.int("dest"), |m| m.int("dest")),
                angle: mv_int("angle"),
                tolerance: mv_int("tolerance"),
                pause: mv_int("pause"),
                retry: mv_int("retry"),
                attempts: mv_int("attempts"),
                timer: mv_int("timer"),
                facing: mv_int("facing"),
                dest_x: mv_int("dest_x"),
                dest_y: mv_int("dest_y"),
                last_x: mv_int("last_x"),
                last_y: mv_int("last_y"),
                coll_x: mv_int("coll_x"),
                coll_y: mv_int("coll_y"),
                orig_x: mv_int("orig_x"),
                orig_y: mv_int("orig_y"),
                off_x: mv_int("off_x"),
                off_y: mv_int("off_y"),
                oxx: grp_int("oxx"),
                whose: grp_int("whose"),
                group_angle: grp_int("group_angle"),
                group_id: grp_int("id"),
                form_id: grp_int("form_id"),
                // `in_group` is `GroupMoveOrder`'s **own** field, past
                // both bases — so on a `GROUPATTACKTOORDER` it sits one
                // block in, on the `GroupMoveOrder` the outer block
                // wraps, and reading it off `o` returned `None` for
                // every grouped attack-move ever captured (item 237).
                in_group: base("GroupMoveOrder").and_then(|g| g.int("in_group")),
                mandatory: atk_int("mandatory"),
                defensive: atk_int("defensive"),
                in_range: atk_int("in_range"),
                ever_in_range: atk_int("ever_in_range"),
                new_ord: atk_int("new_ord"),
                def_x: atk_int("def_x"),
                def_y: atk_int("def_y"),
            }
        })
        .collect()
}

/// The path stack of a `UNITDATA` block: its one `STACK<TYPE>` direct child
/// (§11.1).
///
/// `STACK<TYPE>` is the template's own name, shared by every `Stack<T>`, but
/// a unit writes exactly one of them — the guys are a `PtrArray<Guy>`, whose
/// `length/size/increment` are flat lines on `UNITDATA` itself. An empty
/// stack writes its `BEGIN` and nothing under it, which reads back as no
/// `PATHDATA` children.
pub fn path_of(b: Block<'_>) -> Vec<PathDump> {
    let Some(stack) = b.kid("STACK<TYPE>") else {
        return Vec::new();
    };
    stack
        .kids("PATHDATA")
        .map(|p| PathDump {
            to: (p.int("to_x").unwrap_or(0), p.int("to_y").unwrap_or(0)),
            tolerance: p.int("tolerance").unwrap_or(0),
            flags: p.int("flags").unwrap_or(0),
        })
        .collect()
}

fn unit_of(b: Block<'_>) -> Option<UnitDump> {
    let (flags, o, who, pos) = object_base(b)?;
    let orders = orders_of(b);
    let path = path_of(b);
    let guys = b
        .kids("GUY")
        .map(|g| Guy {
            kind: g.int("type"),
            pos: match (g.int("x"), g.int("y"), g.int("z")) {
                (Some(x), Some(y), Some(z)) => Some(Pos { x, y, z }),
                _ => None,
            },
            angle: g.int("angle"),
            cur_time: g.int("cur_time"),
            end_time: g.int("end_time"),
            last_time: g.int("last_time"),
            cur_anim: g.int("cur_anim"),
            gpiece: g.int("gpiece"),
            guy_flags: g.int("guy_flags"),
            stopped: g.int("stopped"),
            guy_num: g.int("guy_num"),
            hold_attack: g.int("hold_attack"),
            queued_attack: g.int("queued_attack"),
            ox: g.int("ox"),
            whom: g.int("whom"),
            // `des_x`/`des_y` carry no `z`, so the third slot is the
            // figure's own — a `Pos` here is a point, not a placement.
            des: match (g.int("des_x"), g.int("des_y")) {
                (Some(x), Some(y)) => Some(Pos { x, y, z: 0 }),
                _ => None,
            },
            des_angle: g.int("des_angle"),
            track: match (g.int("track_dx"), g.int("track_dy")) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            },
            // `last_x`/`last_y` carry a `last_z` of their own, and it is
            // the figure's, so the third slot is filled where the record
            // has it and left at zero where it does not.
            last_pos: match (g.int("last_x"), g.int("last_y")) {
                (Some(x), Some(y)) => Some(Pos {
                    x,
                    y,
                    z: g.int("last_z").unwrap_or(0),
                }),
                _ => None,
            },
            last_speed: g.int("last_speed"),
            avg_speed: g.int("avg_speed"),
        })
        .collect();
    // `myhits`, `damage` and `damage_frac` sit on the `OBJECT` level, one
    // in from `UNITDATA`'s own fields and one out from `SUBOBJECT`'s —
    // `the_unit_s_hit_points_are_the_object_block_s` pins that against a
    // decoy on either side, which is item 478's lesson one record over.
    let obj = b.find("OBJECT");
    Some(UnitDump {
        flags,
        o,
        who,
        pos,
        angle: b.int("angle"),
        guys,
        orders,
        path,
        group: b.int("group"),
        form: b.int("form"),
        form_mod: b.int("form_mod"),
        stance: b.int("stance"),
        orders_x: b.int("orders_x"),
        orders_y: b.int("orders_y"),
        dest_angle: b.int("dest_angle"),
        tolerance: b.int("tolerance"),
        path_recursion: b.int("path_recursion"),
        idle: b.int("idle"),
        unit_masks: b.int("unit_masks"),
        unit_masks2: b.int("unit_masks2"),
        attrition: b.int("attrition"),
        myspeed: b.int("myspeed"),
        o_up: b.int("o_up"),
        o_down: b.int("o_down"),
        inside_up: b.int("inside_up"),
        myhits: obj.and_then(|o| o.int("myhits")),
        damage: obj.and_then(|o| o.int("damage")),
        damage_frac: obj.and_then(|o| o.int("damage_frac")),
        mylos: obj.and_then(|o| o.int("mylos")),
        infiltrated: obj.and_then(|o| o.int("infiltrated")),
        visible: obj.and_then(|o| o.int("visible")),
        hold_frames: obj.and_then(|o| o.int("hold_frames")),
        collide: b.int("collide"),
        collide_frame: b.int("collide_frame"),
        collide_o: b.int("collide_o"),
        collide_who: b.int("collide_who"),
        collide_guy: b.int("collide_guy"),
        recharging: b.int("recharging"),
        // `UNITDATA`'s own indent, like `recharging` and unlike `damage` —
        // the `OBJECT` block one in carries no `damage_frame` at all, and
        // the animals' nested `UNITDATA` is what puts one an indent
        // further out (§41.1).
        damage_frame: b.int("damage_frame"),
        damage_o: b.int("damage_o"),
        damage_who: b.int("damage_who"),
        safe: b.int("safe"),
        start_dist: b.int("start_dist"),
        uid: obj.and_then(|o| o.int("uid")),
        down: obj.and_then(|o| o.int("down")),
        down_who: obj.and_then(|o| o.int("down_who")),
        up: obj.and_then(|o| o.int("up")),
        up_who: obj.and_then(|o| o.int("up_who")),
    })
}

fn build_of(b: Block<'_>) -> Option<BuildDump> {
    let (flags, o, who, pos) = object_base(b)?;
    let wall = b.kid("WALLDATA");
    // The mining list, pair by pair in file order. `Block` keeps fields in
    // the order they were written, so a `ty` is the partner of the `tx`
    // before it; anything else between them would mean the shape changed.
    let mut gather_from = Vec::new();
    let mut tx: Option<i64> = None;
    for (k, v) in b.fields() {
        match k {
            "tx" => tx = v.trim().parse().ok(),
            "ty" => {
                if let (Some(x), Ok(y)) = (tx.take(), v.trim().parse()) {
                    gather_from.push((x, y));
                }
            }
            _ => {}
        }
    }
    // The `ObjectData` level, one block in from `BUILDDATA` — the same
    // place `unit_of` reads a unit's, and under `WALLDATA` here.
    let obj = b.find("OBJECT");
    Some(BuildDump {
        flags,
        o,
        who,
        pos,
        damage: obj.and_then(|o| o.int("damage")),
        damage_frac: obj.and_then(|o| o.int("damage_frac")),
        orig_type: b.int("orig_type"),
        max_age: b.int("max_age"),
        // **`WALLDATA`'s, not `BUILDDATA`'s** (item 478). The name says
        // `WallData::build_masks` and the record writes it at the wall's
        // own indent, between `ever_seen_completed` and `helpers`; read
        // off the outer block it was `None` on every capture ever taken,
        // so the flag Great Lakes' word turned on was not merely
        // uncompared — it was never parsed. `build_masks_is_the_wall_s_field`.
        build_masks: wall.and_then(|w| w.int("build_masks")),
        gather_from,
        // `length` and `size` sit at `BUILDDATA`'s own indent, between
        // `cliff` and the first `tx`, and nothing else at that level writes
        // either name — the build queue's own count is `queue_size` and is
        // one block down.
        mining_len: b.int("length"),
        mining_size: b.int("size"),
        mtn: b.int("mtn"),
        cliff: b.int("cliff"),
        gather_down: b.int("gather_down"),
        // The clock lives on the `WALLDATA` block, one level in from
        // `BUILDDATA` — `kid`, not `find`, because only the direct child
        // is the building's own.
        job_counter: wall.and_then(|w| w.int("job_counter")),
        constr_time: wall.and_then(|w| w.int("constr_time")),
        construct_hits: wall.and_then(|w| w.int("(int)construct_hits")),
        ever_seen: wall.and_then(|w| w.int("ever_seen")),
        ever_seen_completed: wall.and_then(|w| w.int("ever_seen_completed")),
        queued: b.int("queued"),
        queue: b.kid("BUILDQUEUE").map(queue_of).unwrap_or_default(),
    })
}

/// The slots of a `BUILDQUEUE` block, in file order.
///
/// `BuildQueue::log_data` writes every field of every capacity slot under
/// one key each — `queue[scan].type`, `.job_counter`, `.cost[0..2]`,
/// `.good[0..2]` — so the k-th value of each key is the k-th slot, and the
/// run length is `queue_size`. A slot the log did not reach reads as zero,
/// which is why the count is taken from the shortest run rather than
/// assumed.
fn queue_of(b: Block<'_>) -> Vec<QueueItemDump> {
    let ints = |k: &str| -> Vec<i64> {
        b.all(k)
            .iter()
            .filter_map(|v| v.trim().parse::<i64>().ok())
            .collect()
    };
    let ty = ints("queue[scan].type");
    let jc = ints("queue[scan].job_counter");
    let cost: Vec<Vec<i64>> = (0..3)
        .map(|i| ints(&format!("queue[scan].cost[{i}]")))
        .collect();
    let good: Vec<Vec<i64>> = (0..3)
        .map(|i| ints(&format!("queue[scan].good[{i}]")))
        .collect();
    let n = [ty.len(), jc.len()]
        .into_iter()
        .chain(cost.iter().map(Vec::len))
        .chain(good.iter().map(Vec::len))
        .min()
        .unwrap_or(0);
    (0..n)
        .map(|k| QueueItemDump {
            ty: ty[k],
            job_counter: jc[k],
            cost: [cost[0][k], cost[1][k], cost[2][k]],
            good: [good[0][k], good[1][k], good[2][k]],
        })
        .collect()
}

/// The group pool of one `GAME` or `FRAME` block: the 512 `GROUPDATA`
/// records **directly under `FULL DUMP`**, in file order.
///
/// The hotkey groups are deliberately excluded. They live under
/// `HOTKEYGROUPS`, one `HOTKEYGROUPDATA` wrapper each with a `GROUPDATA`
/// nested inside it, and mixing them in is what makes `priority` look like
/// it takes both values in the same array.
pub fn groups(block: Block<'_>) -> Vec<GroupDump> {
    let body = block.kid("FULL DUMP").unwrap_or(block);
    body.kids("GROUPDATA").map(group_of).collect()
}

/// `GroupsData::last_group[8]` — the one slot per player `get_open_slot`
/// never returns, and the slot `push_group` last installed into.
///
/// It is written **after** the 512 records, at `FULL DUMP`'s own field
/// indent and with no `BEGIN` of its own, so the parser's "a field belongs
/// to the innermost open block" rule hands it to the **last** `GROUPDATA`.
/// Same shape as the leaders' `leader_flags` (see this module's header);
/// re-attached here by position rather than by indentation.
pub fn last_group(block: Block<'_>) -> Vec<i64> {
    let body = block.kid("FULL DUMP").unwrap_or(block);
    let Some(last) = body.kids("GROUPDATA").last() else {
        return Vec::new();
    };
    last.all("last_group")
        .iter()
        .filter_map(|v| v.trim().parse().ok())
        .collect()
}

fn group_of(b: Block<'_>) -> GroupDump {
    let i = |k| b.int(k).unwrap_or(0);
    let col = |k: &str| -> Vec<i64> {
        b.all(k)
            .iter()
            .filter_map(|v| v.trim().parse().ok())
            .collect()
    };
    let (o, off_x, off_y) = (col("list[scan]"), col("off_x[scan]"), col("off_y[scan]"));
    let (cx, cy, ang) = (
        col("curr_x[scan]"),
        col("curr_y[scan]"),
        col("angles[scan]"),
    );
    let members = (0..o.len())
        .map(|k| GroupMemberDump {
            o: o[k],
            off_x: off_x.get(k).copied().unwrap_or_default(),
            off_y: off_y.get(k).copied().unwrap_or_default(),
            curr_x: cx.get(k).copied().unwrap_or_default(),
            curr_y: cy.get(k).copied().unwrap_or_default(),
            angle: ang.get(k).copied().unwrap_or_default(),
        })
        .collect();
    GroupDump {
        id: i("id"),
        who: i("who"),
        num: i("num"),
        army: i("army"),
        ox: i("ox"),
        oy: i("oy"),
        o_dist: i("o_dist"),
        o_angle: i("o_angle"),
        buildings: i("buildings"),
        disband: i("disband"),
        order_num: i("order_num"),
        priority: i("priority"),
        stamp: i("stamp"),
        role: i("role"),
        form: i("form"),
        think_frame: i("think_frame"),
        facing: i("facing"),
        new_speed: i("new_speed"),
        speed: i("speed"),
        form_num: i("form_num"),
        members,
    }
}

fn leader_of(b: Block<'_>) -> LeaderDump {
    let i = |k| b.int(k).unwrap_or(0);
    LeaderDump {
        who: i("who"),
        tribe: i("tribe"),
        defeated_by: i("defeated_by"),
        gov: i("gov"),
        score: i("score"),
        leader_flags: i("leader_flags"),
        leader_flags2: i("leader_flags2"),
        diplos: b
            .all("diplos[scan]")
            .iter()
            .filter_map(|v| v.trim().parse().ok())
            .collect(),
    }
}

fn constants_of<'a>(b: Block<'a>) -> Vec<ConstantDump<'a>> {
    let mut out: Vec<ConstantDump<'a>> = Vec::new();
    for (key, value) in b.fields() {
        let (name, array) = match key.find('[') {
            Some(p) => (&key[..p], true),
            None => (key, false),
        };
        let Ok(v) = value.trim().parse::<i64>() else {
            continue;
        };
        match out.last_mut() {
            Some(last) if last.name == name && array => last.values.push(v),
            _ => out.push(ConstantDump {
                name,
                values: vec![v],
                array,
            }),
        }
    }
    out
}

/// Gathers the per-subsystem records under one block — a `GAME` or a `FRAME`.
/// The object records among a block's **direct** children.
///
/// `before_frames` stops at the first `FRAME` child, which is what the
/// start-of-game state wants: every `FRAME n` is a *child* of `GAME`, and so
/// is the end-of-game `full_dump` that `GameLog::end_game` writes after the
/// last one. Reading all of `GAME`'s children therefore mixes the state at
/// frame 0 with the state at the end — 27 buildings where the game began
/// with 13, and on a long fulldump 400 "citizens" where there were 5. That
/// mis-read is invisible in a position diff (the extra units are never
/// matched to a logged one) and showed up only in the order diff, as ten
/// starting citizens whose derived order was `None` because the duplicate
/// links took the assignment.
pub(crate) fn records(
    b: Block<'_>,
    before_frames: bool,
) -> (Vec<UnitDump>, Vec<BuildDump>, Vec<LeaderDump>) {
    let stop = if before_frames {
        b.children()
            .position(|c| c.name().starts_with("FRAME"))
            .unwrap_or(b.children().len())
    } else {
        b.children().len()
    };
    records_range(b, 0, stop)
}

/// [`records`] over a chosen slice of a block's children, which is what the
/// end-of-game state needs: `GameLog::end_game`'s dump is written at
/// `FRAME`'s own indent *after* the last frame, so it is a run of `GAME`'s
/// trailing children rather than a block of its own ([`Log::final_state`]).
pub(crate) fn records_range(
    b: Block<'_>,
    start: usize,
    stop: usize,
) -> (Vec<UnitDump>, Vec<BuildDump>, Vec<LeaderDump>) {
    // A `DUMP_ALL` dump nests each state under a `FULL DUMP` block — the
    // start-of-game state under the first of `GAME`'s two (the second is
    // `begin_frame(0)`'s, the same state), and engine frame `n − 1`'s end
    // under `FRAME n`'s one (`docs/SYNC.md` §1; `frame_seeds` reads the
    // same block) — so the object lists are that block's children and the
    // `leader_flags` run is on its fields. Any other dump writes them on
    // `GAME` and `FRAME` directly.
    let slice = || b.children().head(stop).tail(start);
    // The `leader_flags` pairs are the *parent's* flat fields, in document
    // order over the whole block, so a slice that starts past some
    // `LEADERDATA` children has to index the run from there — otherwise the
    // end-of-game leaders take the start dump's flags. A `FULL DUMP` body
    // carries its own run and starts at nought.
    let mut flag_skip = b
        .children()
        .head(start)
        .filter(|c| c.name() == "LEADERDATA")
        .count();
    // `find` consumes the iterator it is called on, so the `None` arm takes
    // a **fresh** slice — reusing the exhausted one hands every caller an
    // empty dump, which is what the first draft of this did.
    let (b, kids): (Block<'_>, Children<'_>) = match slice().find(|c| c.name() == "FULL DUMP") {
        Some(dump) => {
            flag_skip = 0;
            (dump, dump.children())
        }
        None => (b, slice()),
    };
    // Gaia's animals are written as `ANIMALDATA` → `UNITDATA` (the
    // `AnimalData::log_data` wrapper adds `ox`, `whom`, `aid` after the
    // unit), in the leader-8 run after every player's units.
    let units = unit_blocks(kids).filter_map(unit_of).collect();
    let builds = kids
        .filter(|c| c.name() == "BUILDDATA")
        .filter_map(build_of)
        .collect();
    let mut leaders: Vec<LeaderDump> = kids
        .filter(|c| c.name() == "LEADERDATA")
        .map(leader_of)
        .collect();
    // `Leaders::log_data` writes each leader's `leader_flags` pair **before**
    // its `BEGIN LEADERDATA` — the first pair in a dump precedes the first
    // block, and the last block has none after it. The trailing-run rule
    // therefore hands each block the *next* leader's flags (off by one), and
    // the parse comment's LEADERDATA example had the direction backwards.
    // The parent block accumulates every pair in document order (the
    // both-candidates rule), so the correct assignment is a zip by index.
    let run = |key: &str| -> Vec<i64> {
        b.fields()
            .filter(|(k, _)| *k == key)
            .filter_map(|(_, v)| v.trim().parse().ok())
            .collect()
    };
    let (flags, flags2) = (run("leader_flags"), run("leader_flags2"));
    for (i, l) in leaders.iter_mut().enumerate() {
        if let Some(&f) = flags.get(i + flag_skip) {
            l.leader_flags = f;
        }
        if let Some(&f) = flags2.get(i + flag_skip) {
            l.leader_flags2 = f;
        }
    }
    (units, builds, leaders)
}

// Keep the record ordering shared: direct units first, then animal wrappers.
fn unit_blocks(kids: Children<'_>) -> impl Iterator<Item = Block<'_>> {
    kids.filter(|c| c.name() == "UNITDATA").chain(
        kids.filter(|c| c.name() == "ANIMALDATA")
            .filter_map(|a| a.kid("UNITDATA")),
    )
}

pub(crate) fn observation_rows(b: Block<'_>, collect_bodies: bool) -> Vec<FrameUnit> {
    let body = b.kid("FULL DUMP").unwrap_or(b);
    unit_blocks(body.children())
        .filter(|u| {
            // Match Guy::has_clock using the same integer field accessor as
            // unit_of. Retain the whole unit (including unclocked figures)
            // when any figure carries a clock.
            collect_bodies
                || u.kids("GUY").any(|g| {
                    g.int("cur_time").is_some()
                        && g.int("end_time").is_some()
                        && g.int("cur_anim").is_some()
                })
        })
        .filter_map(unit_of)
        .map(|u| {
            let goal = u.orders_front_first().next().and_then(|o| {
                Some(Pos {
                    x: o.x?,
                    y: o.y?,
                    z: 0,
                })
            });
            FrameUnit {
                who: u.who,
                o: u.o,
                orderless: u.orders.is_empty(),
                pos: u.pos,
                goal,
                guys: u.guys,
            }
        })
        .collect()
}

/// `Farms::log_data`'s list off the enclosing block's flat fields: one
/// record per `farm_type`, taking the `valid` immediately before it and the
/// nearest `who`/`o` pair before that. Every other `who`/`o` on the block
/// (the leaders' run, the ambience) is left alone because none of them is
/// followed by a `farm_type` without an intervening pair.
pub(crate) fn farms_of(b: Block<'_>) -> Vec<FarmDump> {
    let int = |v: &str| v.trim().parse::<i64>().ok();
    let mut out = Vec::new();
    let (mut who, mut o, mut valid) = (None, None, None);
    let (mut status, mut adds): (Vec<i64>, Vec<i64>) = (Vec::new(), Vec::new());
    for (k, v) in b.fields() {
        match k {
            "who" => who = int(v),
            "o" => o = int(v),
            "valid" => valid = int(v),
            "status[scan][scan2]" => status.push(int(v).unwrap_or(0)),
            "percent[scan][scan2]" => {
                // The float is a count of `0.005f` adds; six decimals over
                // a five-thousandth step names the count exactly.
                let p: f64 = v.trim().parse().unwrap_or(0.0);
                let n = (p / 0.005).round();
                assert!(
                    (p / 0.005 - n).abs() < 0.1,
                    "a farm cell's percent is not a whole number of 0.005f adds: {v}"
                );
                adds.push(n as i64);
            }
            "farm_type" => {
                if let (Some(w), Some(oo)) = (who, o) {
                    out.push(FarmDump {
                        who: w,
                        o: oo,
                        valid: valid.unwrap_or(0),
                        farm_type: int(v).unwrap_or(0),
                        status: std::mem::take(&mut status),
                        adds: std::mem::take(&mut adds),
                    });
                }
                // Each record carries its own pair; a `farm_type` with no
                // fresh one is the list's trailing slot, not a farm.
                who = None;
                o = None;
                valid = None;
                status.clear();
                adds.clear();
            }
            _ => {}
        }
    }
    out
}

impl<'a> Log<'a> {
    /// The start-of-game state, from `GAME INFO` and the body of `GAME`
    /// before the first frame.
    pub fn initial(&'a self) -> Option<Initial<'a>> {
        self.initial_with_bodies(true)
    }

    /// Replay setup keeps all seeding and animation inputs, but not the
    /// figure-position audit series. `build_sim` does not consume that series;
    /// audits must continue to use `initial`.
    pub(crate) fn replay_initial(&'a self) -> Option<Initial<'a>> {
        self.initial_with_bodies(false)
    }

    fn initial_with_bodies(&'a self, collect_bodies: bool) -> Option<Initial<'a>> {
        let game = self.game()?;
        let mut init = Initial::default();
        if let Some(gi) = self.root("GAME INFO").and_then(|g| g.kid("GAMEINFO")) {
            init.game_info = gi.fields().to_vec();
            init.players = gi.kids("PLAYER").map(|p| p.fields().to_vec()).collect();
        }
        // A `DUMP_ALL` dump writes the start-of-game state under `GAME`'s
        // first `FULL DUMP` rather than on `GAME` itself (`records` takes the
        // object lists from the same place). Its `WorldData::log_data` runs
        // at the override detail, so the cells and tiles are there whatever
        // `[Start Game] WORLD` said — run20 is read this way.
        let body = game.kid("FULL DUMP").unwrap_or(game);
        if let Some(w) = body.kid("WORLD") {
            init.world = w.fields().to_vec();
        }
        if let Some(c) = body.kid("CITIES") {
            init.cities = c.kids("CITY").map(city_of).collect();
        }
        if let Some(k) = body.kid("CONSTANTS") {
            init.constants = constants_of(k);
        }
        let (units, builds, leaders) = records(game, true);
        init.units = units;
        init.builds = builds;
        init.leaders = leaders;
        init.checksums = self.checksums();
        // **The start-of-game state is read from the start of the game.**
        // These four used to be whole-log searches, which on a capture with
        // no `DUMP_ALL` head walked every frame to answer `None` — and
        // where one *did* answer, it answered with a mid-game block's
        // regions or heights as though they were the opening state. The
        // head is `GAME`'s children before its first `FRAME`, which is what
        // `records(game, true)` already takes.
        let kids = game.children();
        let head = kids.head(
            kids.clone()
                .position(|c| c.name().starts_with("FRAME"))
                .unwrap_or(kids.len()),
        );
        let at_head = |name: &str| head.into_iter().find_map(|c| find_here(c, name));
        init.heights = head.into_iter().find_map(heights_in).unwrap_or_default();
        if let Some(r) = at_head("REGIONS") {
            init.regions = r
                .kids("REGION")
                .map(|b| RegionDump {
                    region: b.int("region").unwrap_or(-1),
                    flags: b.int("flags").unwrap_or(0),
                    size: b.int("size").unwrap_or(0),
                })
                .collect();
        }
        if let Some(h) = at_head("HERDS") {
            init.herds = h
                .kids("HERD")
                .map(|b| HerdDump {
                    cx: b.int("cx").unwrap_or(0),
                    cy: b.int("cy").unwrap_or(0),
                    wx: b.int("wx").unwrap_or(0),
                    wy: b.int("wy").unwrap_or(0),
                    t: b.int("t").unwrap_or(0),
                    herd_flags: b.int("herd_flags").unwrap_or(0),
                })
                .collect();
        }
        // `Good::log_data@0066e610` writes the type's name as a bare line
        // and then the `SubObject` base, so the name is the record's one
        // field with no value.
        if let Some(d) = at_head("FULL DUMP") {
            init.goods = d
                .kids("GOOD")
                .map(|b| {
                    let sub = b.kid("SUBOBJECT");
                    GoodDump {
                        name: b
                            .fields()
                            .find(|(_, v)| v.is_empty())
                            .map_or("", |(k, _)| k),
                        o: sub.and_then(|s| s.int("o")).unwrap_or(-1),
                        x: sub.and_then(|s| s.int("x_internal")).unwrap_or(0),
                        y: sub.and_then(|s| s.int("y_internal")).unwrap_or(0),
                        flags: sub.and_then(|s| s.int("flags")).unwrap_or(0),
                    }
                })
                .collect();
        }
        init.farms = farms_of(body);
        // **One walk of the frames, four products** — the seeds, the
        // animation lengths, the clocks the harness installs, and the
        // figures' own positions, which it compares. The two figure filters
        // are different (a dump can print a `GUY` block's position without
        // its clock), and since item 260 the walk *re-reads* the capture
        // rather than crossing an arena that is already built, so doing it
        // four times over is four parses of a 790 MB file rather than four
        // pointer chases.
        self.append_observations(&mut init, collect_bodies);
        Some(init)
    }

    /// Append owned observation products; none borrow the chunk being scanned.
    pub(crate) fn append_observations(&self, init: &mut Initial<'_>, collect_bodies: bool) {
        for r in self.roots() {
            if Some(r.node) == self.game_node() {
                continue;
            }
            anims_in(r, &mut init.anim_lengths);
        }
        self.scan_children(|b| {
            anims_in(b, &mut init.anim_lengths);
            let Some(n) = frame_number(b) else {
                return true;
            };
            if let Some(dump) = b.kid("FULL DUMP")
                && let Some(c) = checksums_in(dump.fields()).first()
            {
                init.frame_seeds.push((n - 1, c.seed));
            }
            let rows = observation_rows(b, collect_bodies);
            if !collect_bodies {
                let clocked: Vec<FrameUnit> = rows
                    .into_iter()
                    .filter(|u| u.guys.iter().any(Guy::has_clock))
                    .collect();
                if !clocked.is_empty() {
                    init.frame_guys.push((n - 1, clocked));
                }
                return true;
            }
            let clocked: Vec<FrameUnit> = rows
                .iter()
                .filter(|u| u.guys.iter().any(Guy::has_clock))
                .cloned()
                .collect();
            if !clocked.is_empty() {
                init.frame_guys.push((n - 1, clocked));
            }
            let bodied: Vec<FrameUnit> = rows
                .into_iter()
                .filter(|u| u.guys.iter().any(|g| g.pos.is_some()))
                .collect();
            if !bodied.is_empty() {
                init.frame_bodies.push((n - 1, bodied));
            }
            true
        });
        init.anim_lengths.sort_unstable();
        init.anim_lengths.dedup();
    }

    /// Every `(gpiece, cur_anim, end_time)` the dump's `GUY` blocks show,
    /// with `end_time > 0`, deduplicated and sorted — every state the dump
    /// printed, the frames included. A `DUMP_ALL` dump only; the lengths
    /// are the animation packets' frame counts (`docs/ANIM.md` §3).
    pub fn anim_lengths(&'a self) -> Vec<(i64, i64, i64)> {
        let walk = anims_in;
        let mut out = Vec::new();
        let game = self.game_node();
        for r in self.roots() {
            if Some(r.node) == game {
                continue;
            }
            walk(r, &mut out);
        }
        self.scan_children(|b| {
            walk(b, &mut out);
            true
        });
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Every frame's typed state, in order.
    /// The state `GameLog::end_game` writes on the way out, as a [`Frame`]
    /// labelled with the run's last frame.
    ///
    /// **Every capture that quits has one, and nothing had ever read it.**
    /// `!quit` leaves the last `FRAME n` blocks empty and the shutdown dump
    /// lands *after* them at `FRAME`'s own indent — a run of `GAME`'s
    /// trailing children, which the frame walk cannot reach and
    /// [`Self::dumps`] catches only when it is wrapped in a `FULL DUMP`
    /// (the `DUMP_ALL` shape). On an ordinary windowed capture it is not,
    /// so the whole map's positions at the quit frame sat unparsed: run82's
    /// **6946** is seventeen frames past the last block anyone had read of
    /// East Indies, and inside the band item 241 was booked to buy.
    ///
    /// `None` when the trailing run holds no object record — which is every
    /// `DUMP_ALL` capture (theirs is a `FULL DUMP`, and [`Self::dumps`]
    /// already has it) and every log that was killed rather than quit.
    pub fn final_state(&'a self) -> Option<Frame> {
        let game = self.game()?;
        let kids = game.children();
        let last = kids
            .enumerate()
            .filter(|(_, c)| c.name().starts_with("FRAME "))
            .last()?;
        let n: i64 = last.1.name().strip_prefix("FRAME ")?.trim().parse().ok()?;
        let start = last.0 + 1;
        if start >= kids.len() {
            return None;
        }
        let (units, builds, leaders) = records_range(game, start, kids.len());
        if units.is_empty() && builds.is_empty() && leaders.is_empty() {
            return None;
        }
        let cities = kids
            .tail(start)
            .find(|c| c.name() == "CITIES")
            .map(|c| c.kids("CITY").map(city_of).collect())
            .unwrap_or_default();
        Some(Frame {
            n,
            units,
            builds,
            leaders,
            cities,
            deaths: kids
                .tail(start)
                .filter(|c| c.name() == "DEATH_OBJS")
                .map(death_of)
                .collect(),
        })
    }

    /// Decode ordinary frames in source order, retaining only the current frame
    /// unless the visitor keeps it. The limit counts records, not frame labels.
    /// A zero limit does not scan children; reaching the limit stops before the
    /// next child is decoded. This does not change `initial`'s whole-log scan.
    pub fn visit_frame_states(&self, limit: Option<usize>, mut visit: impl FnMut(Frame)) {
        let mut remaining = limit.unwrap_or(usize::MAX);
        if remaining == 0 {
            return;
        }
        self.scan_children(|b| {
            if let Some(n) = frame_number(b) {
                let (units, builds, leaders) = records(b, false);
                let cities = b
                    .find("CITIES")
                    .map(|c| c.kids("CITY").map(city_of).collect())
                    .unwrap_or_default();
                visit(Frame {
                    n,
                    units,
                    builds,
                    leaders,
                    cities,
                    deaths: deaths_of(b),
                });
                remaining -= 1;
            }
            remaining != 0
        });
    }

    /// Every frame's typed state, in order. Use `visit_frame_states` for a
    /// bounded consumer rather than retaining the entire decoded sequence.
    pub fn frame_states(&'a self) -> Vec<Frame> {
        let mut out = Vec::new();
        self.visit_frame_states(None, |frame| out.push(frame));
        out
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_clock_rows_match_complete_records() {
        let unit = |id, clock: &str| {
            format!(
                "BEGIN UNITDATA\n BEGIN SUBOBJECT\n  flags 1\n  o {id}\n  who 0\n BEGIN GUY\n  x 1\n  y 2\n  z 3\n{clock} BEGIN GUY\n  type 99\n"
            )
        };
        let indent = |text: &str| text.lines().map(|l| format!(" {l}\n")).collect::<String>();
        let valid = "  cur_time 0\n  end_time 0\n  cur_anim 0\n";
        let body = format!(
            "{}{}{}{}{}",
            indent(&unit(1, "")),
            indent(&unit(2, valid)),
            indent(&unit(3, "  cur_time bad\n  end_time 1\n  cur_anim 2\n")),
            indent(&unit(4, "  cur_time 1\n  end_time 2\n")),
            format_args!(" BEGIN ANIMALDATA\n{}", indent(&indent(&unit(5, valid))))
        );
        for nested in [false, true] {
            let text = if nested {
                format!("BEGIN FRAME 7\n BEGIN FULL DUMP\n{}", indent(&body))
            } else {
                format!("BEGIN FRAME 7\n{body}")
            };
            let log = Log::parse(&text);
            let frame = log.roots().next().unwrap();
            let expected: Vec<_> = records(frame, false)
                .0
                .into_iter()
                .filter(|u| u.guys.iter().any(Guy::has_clock))
                .map(|u| FrameUnit {
                    who: u.who,
                    o: u.o,
                    orderless: true,
                    pos: u.pos,
                    goal: None,
                    guys: u.guys,
                })
                .collect();
            let actual = observation_rows(frame, false);
            assert_eq!(actual, expected);
            assert_eq!(actual.iter().map(|u| u.o).collect::<Vec<_>>(), [2, 5]);
            assert!(actual.iter().all(|u| u.guys.len() == 2));
            assert_eq!(observation_rows(frame, true).len(), 5);
        }
    }

    #[test]
    fn replay_initial_preserves_every_correction_input() {
        let Some(path) = crate::testenv::dump("gamelog-run13-window-95-105.txt") else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap();
        let log = Log::parse(&text);
        let mut full = log.initial().unwrap();
        assert!(!full.frame_seeds.is_empty());
        assert!(!full.frame_guys.is_empty());
        assert!(!full.frame_bodies.is_empty());
        full.frame_bodies.clear();
        assert_eq!(log.replay_initial().unwrap(), full);
    }

    #[test]
    fn frame_visitor_matches_direct_blocks_and_limits_by_record() {
        let text = "BEGIN GAME\n BEGIN FRAME 9\n  BEGIN FULL DUMP\n   BEGIN CITIES\n    BEGIN CITY\n     who 1\n     o 2000\n BEGIN FRAME 9\n BEGIN FRAME 12\n  BEGIN CITIES\n   BEGIN CITY\n    who 0\n    o 2001\n";
        let reference = Log::parse(text);
        let expected: Vec<_> = reference
            .frames()
            .into_iter()
            .map(|(n, b)| {
                let (units, builds, leaders) = records(b, false);
                let cities = b
                    .find("CITIES")
                    .map(|c| c.kids("CITY").map(city_of).collect())
                    .unwrap_or_default();
                Frame {
                    n,
                    units,
                    builds,
                    leaders,
                    cities,
                    deaths: deaths_of(b),
                }
            })
            .collect();
        assert_eq!(
            expected.iter().map(|f| f.n).collect::<Vec<_>>(),
            vec![9, 9, 12]
        );
        for limit in [0, 1, 2, 3, 8] {
            let log = Log::parse(text);
            let mut actual = Vec::new();
            log.visit_frame_states(Some(limit), |f| actual.push(f));
            assert_eq!(actual, expected[..limit.min(expected.len())]);
        }
        let log = Log::parse(text);
        assert_eq!(log.frame_states(), expected);
        Log::parse("BEGIN GAME\n").visit_frame_states(None, |_| panic!("invented a frame"));
    }

    // Every snippet below is the shape of real lines from this install's
    // logs, trimmed; see the module note for where they come from.
    const SAMPLE: &str = "\
play2, team 0 8
init_teams: on_team 0
BEGIN GAME INFO
 RUN COUNT 0
 Player
 WHO =  0
 BEGIN GAMEINFO
  (int) version 111935788
  MAP_SIZE 2
  (int)seed 12345
  BEGIN PLAYER
   flags 7
   tribe 11
   who 0
   Player
  BEGIN PLAYER
   flags 1
   tribe 2
   who 1
   Emperor Huayna Capac
BEGIN GAME
 BEGIN COMMANDMANAGER
 BEGIN WORLD
  seed 7236
  xs 60
  ys 60
  player_territory_limit 44
 BEGIN CITIES
  length 20
  BEGIN CITY
   x 3168
   y 30816
   pop 1
   who 0
 BEGIN BUILDDATA
  BEGIN WALLDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 39
     o 2000
     who 0
     x_internal 3168
     y_internal 30816
     z_internal 536
  city 0
  orig_type 418
  BEGIN BUILDQUEUE
   queue_size 2
   queue[scan].type -1
  mtn -1
  cliff -1
  length 3
  size 160
  increment -1
  flags 0
  tx 20
  ty 146
  tx 21
  ty 147
  tx 22
  ty 148
  length 0
 BEGIN CONSTANTS
  unit_move_speed 1
  rocky_modifier 170
  fort_upgrade_terr[scan] 2
  fort_upgrade_terr[scan] 4
  fort_upgrade_terr[scan] 6
  fort_upgrade_terr[scan] 9
  units_killed[scan2] 0
  units_killed[scan2] 3
  one_age_down 15
 BEGIN UNITDATA
  BEGIN OBJECT
   BEGIN SUBOBJECT
    flags 65
    o 0
    who 0
    x_internal 4248
    y_internal 32664
    z_internal 528
  BEGIN GUY
   type 69
   x 4248
   y 32664
   z 539
   angle 1431655765
  BEGIN GUY
   type 69
   x 4254
   y 32555
   z 548
   angle 1431655765
 leader_flags 176160775
 leader_flags2 0
 BEGIN LEADERDATA
  who 0
  tribe 11
  defeated_by -1
  gov -1
  score 0
 leader_flags 176160787
 leader_flags2 0
 BEGIN LEADERDATA
  who 1
  tribe 2
  defeated_by -1
  gov -1
  score 0
 BEGIN GUY
  type 69
  x 4248
  y 32664
  z 539
  angle 1431655765
 BEGIN FRAME 1
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 73
     o 0
     who 0
     x_internal 4248
     y_internal 32664
     z_internal 528
   BEGIN GUY
   BEGIN GUY
  leader_flags 33554451
  leader_flags2 0
  BEGIN LEADERDATA
   who 0
   tribe 11
   defeated_by -1
   gov -1
   score 181
 BEGIN FRAME 2
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 73
     o 0
     who 0
     x_internal 4250
     y_internal 32664
     z_internal 528
   BEGIN GUY
   BEGIN GUY
 BEGIN BUILDDATA
  BEGIN WALLDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 39
     o 2007
     who 0
     x_internal 100
     y_internal 200
     z_internal 0
 BEGIN UNITDATA
  BEGIN OBJECT
   BEGIN SUBOBJECT
    flags 65
    o 9
    who 0
    x_internal 1
    y_internal 2
    z_internal 3
";

    /// **The shutdown dump is a sibling, and the frame walk cannot see
    /// it.** `!quit` closes the last `FRAME n` block empty and
    /// `GameLog::end_game` then writes the whole map at `FRAME`'s own
    /// indent, so on every windowed capture the last state on disk is a
    /// run of `GAME`'s trailing children. `SAMPLE`'s tail is that shape —
    /// a `BUILDDATA` and a `UNITDATA` after `FRAME 2` — and
    /// [`Log::final_state`] is the only reader of it.
    #[test]
    fn the_shutdown_dump_is_a_frame_the_frame_walk_cannot_reach() {
        let log = Log::parse(SAMPLE);
        // The frame walk itself sees nothing of it.
        let states = log.frame_states();
        assert_eq!(
            states.iter().map(|f| f.n).collect::<Vec<_>>(),
            vec![1, 2],
            "SAMPLE's frames"
        );
        assert!(
            states.iter().all(|f| !f.units.iter().any(|u| u.o == 9)),
            "the trailing unit is nobody's frame child"
        );
        // `final_state` reads it, labelled with the run's last frame.
        let fin = log.final_state().expect("SAMPLE quits after FRAME 2");
        assert_eq!(fin.n, 2, "the shutdown dump takes the last frame's label");
        assert_eq!(
            fin.units.iter().map(|u| (u.who, u.o)).collect::<Vec<_>>(),
            vec![(0, 9)],
            "the trailing UNITDATA, and only it"
        );
        assert_eq!(
            fin.builds.iter().map(|b| (b.who, b.o)).collect::<Vec<_>>(),
            vec![(0, 2007)],
            "the trailing BUILDDATA with it"
        );
        // And it is `None` rather than a duplicate when the last frame is
        // the last thing in the file — which is every capture that was
        // killed rather than quit. Cutting SAMPLE at the frame's end is
        // how this was made to fail.
        let cut = &SAMPLE[..SAMPLE.rfind(" BEGIN BUILDDATA\n").unwrap()];
        let cut = Log::parse(cut);
        assert_eq!(
            cut.frame_states().iter().map(|f| f.n).collect::<Vec<_>>(),
            vec![1, 2],
            "the cut keeps both frames — otherwise the None below is vacuous"
        );
        assert!(
            cut.final_state().is_none(),
            "no trailing run, no final state"
        );
    }

    /// Every block of one tree, as a string: the shape a lazy read has to
    /// reproduce exactly.
    #[cfg(test)]
    fn spell(b: Block<'_>, depth: usize, out: &mut String) {
        out.push_str(&format!("{:indent$}[{}]\n", "", b.name(), indent = depth));
        for (k, v) in b.fields() {
            out.push_str(&format!("{:indent$}{k}={v}\n", "", indent = depth + 1));
        }
        for c in b.children() {
            spell(c, depth + 1, out);
        }
    }

    #[cfg(test)]
    fn spelled(log: &Log<'_>) -> String {
        let mut out = String::new();
        for (k, v) in &log.preamble {
            out.push_str(&format!("!{k}={v}\n"));
        }
        for r in log.roots_here() {
            spell(r, 0, &mut out);
        }
        out
    }

    /// **The lazy read is the eager read, block for block and field for
    /// field.** Item 260 stops [`Log::parse`] at the first `FRAME` and
    /// indexes the rest; every rule the eager pass has — the trailing run
    /// at a block's own indent, the `leader_flags` written before the block
    /// they describe, a field belonging to two blocks at once — has to
    /// survive being applied to one frame's text in isolation. This is the
    /// check that says it does, and it is the reason the refactor is
    /// reversible: made to fail by indexing a span that stops at its last
    /// child rather than at the next `BEGIN`, which drops every trailing
    /// field, it reports the first block that differs.
    #[test]
    fn the_lazy_tree_is_the_eager_tree() {
        // A tail with every shape the index has to model: a field at the
        // frames' own indent (the `leader_flags` quirk, which belongs to
        // the parent *and* to the frame it follows), a frame whose last
        // line is a field rather than a block, and a sibling of `FRAME`
        // that is not one.
        let quirky = "BEGIN GAME\n BEGIN WORLD\n  xs 60\n BEGIN FRAME 1\n  BEGIN UNITDATA\n   o 0\n  who 1\n  tail 7\n BEGIN FRAME 2\n  BEGIN UNITDATA\n   o 1\n flag 9\n flag2 10\n BEGIN LEADERDATA\n  who 0\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   o 2\n";
        for (what, text) in [("the sample", SAMPLE), ("the quirky tail", quirky)] {
            let lazy = Log::parse(text);
            let eager = Log::parse_eager(text);
            assert!(
                lazy.indexed() >= 3,
                "{what} indexed {} blocks — the comparison is vacuous",
                lazy.indexed()
            );
            assert_eq!(spelled(&lazy), spelled(&eager), "{what} reads differently");
        }
        // And on a real capture, where the writers' inconsistencies are.
        let Some(path) = crate::testenv::dump("gamelog-run87-greatlakes-blockedwalker.txt") else {
            eprintln!("skipping the capture half: no dumps (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        // The first frames only: the whole file spells to gigabytes, and
        // the shapes this has to get right are all in the head and the
        // first frame boundary.
        let cut = text.find(" BEGIN FRAME ").expect("a capture has frames");
        let after = text[cut + 1..]
            .match_indices(" BEGIN FRAME ")
            .nth(3)
            .map_or(text.len(), |(i, _)| cut + 1 + i);
        let text = &text[..after];
        let lazy = Log::parse(text);
        let eager = Log::parse_eager(text);
        assert!(
            lazy.frames().len() >= 3 && lazy.indexed() >= 3,
            "the cut kept {} frames and indexed {} blocks — the comparison \
             would be vacuous",
            lazy.frames().len(),
            lazy.indexed()
        );
        assert_eq!(
            spelled(&lazy),
            spelled(&eager),
            "run87 reads differently lazily"
        );
    }

    /// The arena's widths are the whole of item 235, and prose does not
    /// hold them: a `Block` that grows a `Vec` back, or a `Field` that
    /// goes back to fat pointers, doubles the suite's peak in silence.
    /// 76.6 M fields on one capture is what makes each of these bytes
    /// worth 76.6 MB.
    #[test]
    fn the_arena_s_widths_are_what_item_235_bought() {
        assert_eq!(size_of::<Field>(), 16, "four u32 offsets, not two &str");
        assert_eq!(size_of::<Block<'_>>(), 16, "a cursor: a &Log and an index");
        assert!(
            size_of::<Node>() <= 32,
            "a node is ranges, not owned vectors: {}",
            size_of::<Node>()
        );
        // And the chunks are one size, which is what lets a log reuse the
        // pages the log before it gave back.
        let mut c: Chunks<u32> = Chunks::default();
        for i in 0..(CHUNK as u32 + 3) {
            c.push(i);
        }
        assert_eq!(c.len(), CHUNK + 3);
        assert_eq!(c.chunks.len(), 2, "one chunk per CHUNK entries, exactly");
        assert!(c.chunks.iter().all(|k| k.len() == CHUNK));
        assert_eq!(c.get(0), 0);
        assert_eq!(c.get(CHUNK - 1), CHUNK as u32 - 1);
        assert_eq!(c.get(CHUNK + 2), CHUNK as u32 + 2);
    }

    #[test]
    fn preamble_and_roots() {
        let log = Log::parse(SAMPLE);
        assert_eq!(log.preamble[0], ("play2,", "team 0 8"));
        assert_eq!(log.preamble[1], ("init_teams:", "on_team 0"));
        let names: Vec<_> = log.roots().map(|b| b.name()).collect();
        assert_eq!(names, vec!["GAME INFO", "GAME"]);
    }

    #[test]
    fn blocks_nest_by_indent_and_close_on_dedent() {
        let log = Log::parse(SAMPLE);
        let game = log.game().unwrap();
        let names: Vec<_> = game.children().map(|b| b.name()).collect();
        assert_eq!(
            names,
            vec![
                "COMMANDMANAGER",
                "WORLD",
                "CITIES",
                "BUILDDATA",
                "CONSTANTS",
                "UNITDATA",
                "LEADERDATA",
                "LEADERDATA",
                "GUY",
                "FRAME 1",
                "FRAME 2",
                // The end-of-game `full_dump`, at the same depth as the
                // frames and after them.
                "BUILDDATA",
                "UNITDATA"
            ]
        );
        // The building's SUBOBJECT is three levels down.
        let sub = game.kid("BUILDDATA").unwrap().find("SUBOBJECT").unwrap();
        assert_eq!(sub.indent(), 4);
        assert_eq!(sub.int("o"), Some(2000));
    }

    #[test]
    fn fields_attach_to_the_innermost_open_block_whatever_their_indent() {
        // The leaders writer puts each `leader_flags` pair *before* its
        // block, so at the raw block layer the trailing-run rule hands who-0
        // its successor's value and who-1 (last, no pair after) nothing —
        // while the parent accumulates the true ordered run. `records`' zip
        // (asserted in `the_initial_state_is_extracted`) is what un-skews it.
        let log = Log::parse(SAMPLE);
        let game = log.game().unwrap();
        let leaders: Vec<_> = game.kids("LEADERDATA").collect();
        assert_eq!(leaders.len(), 2);
        assert_eq!(leaders[0].int("leader_flags"), Some(176160787));
        assert_eq!(leaders[1].int("leader_flags"), None);
        let run: Vec<_> = game
            .fields()
            .filter(|(k, _)| *k == "leader_flags")
            .map(|(_, v)| v)
            .collect();
        assert_eq!(run, ["176160775", "176160787"]);
    }

    #[test]
    fn labels_without_values_are_kept_as_empty_fields() {
        let log = Log::parse(SAMPLE);
        let gi = log.root("GAME INFO").unwrap();
        assert_eq!(gi.get("Player"), Some(""));
        assert_eq!(gi.get("WHO"), Some("=  0"));
        let players: Vec<_> = gi.kid("GAMEINFO").unwrap().kids("PLAYER").collect();
        assert_eq!(players[1].get("Emperor"), Some("Huayna Capac"));
    }

    #[test]
    fn constants_group_array_entries_under_one_name() {
        let log = Log::parse(SAMPLE);
        let init = log.initial().unwrap();
        let k = &init.constants;
        assert_eq!(k[0].name, "unit_move_speed");
        assert_eq!(k[0].values, vec![1]);
        assert!(!k[0].array);
        assert_eq!(k[1].name, "rocky_modifier");
        assert_eq!(k[1].values, vec![170]);
        assert_eq!(k[2].name, "fort_upgrade_terr");
        assert_eq!(k[2].values, vec![2, 4, 6, 9]);
        assert!(k[2].array);
        assert_eq!(k[3].name, "units_killed");
        assert_eq!(k[3].values, vec![0, 3]);
        assert_eq!(k[4].name, "one_age_down");
        assert_eq!(k.len(), 5);
    }

    /// **`build_masks` is `WallData`'s field, at the wall's own indent**
    /// (item 478).
    ///
    /// The record writes it between `ever_seen_completed` and `helpers`,
    /// one level *in* from `BUILDDATA` — and this reader took it off the
    /// outer block, so it was `None` on every capture ever taken and the
    /// diff had nothing to compare. `0x100` is the replan flag
    /// (`docs/ROADS.md` §1), and it is what held Great Lakes' word at
    /// 10234 for four items while sitting in plain sight in every block.
    ///
    /// **Made to fail on purpose**: the fixture puts a decoy `build_masks`
    /// at `BUILDDATA`'s own indent as well, so the old `b.int` reading
    /// returns `7` here and this test reddens on the value rather than on
    /// `None` — a reader that goes back to the outer block cannot pass by
    /// accident on a capture that happens not to have one.
    #[test]
    fn build_masks_is_the_wall_s_field() {
        const TEXT: &str = "\
BEGIN GAME
 BEGIN WORLD
  seed 1
 BEGIN BUILDDATA
  BEGIN WALLDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 7
     o 2006
     who 0
     x_internal 2688
     y_internal 29568
     z_internal 0
   ever_seen_completed 3
   build_masks 4352
   helpers 0
  city 0
  build_masks 7
  orig_type 436
 BEGIN FRAME 1
";
        let log = Log::parse(TEXT);
        let init = log.initial().expect("the sample has a start block");
        assert_eq!(init.builds.len(), 1);
        assert_eq!(
            init.builds[0].build_masks,
            Some(4352),
            "build_masks came off `BUILDDATA` rather than `WALLDATA`"
        );
        assert_eq!(init.builds[0].build_masks.unwrap() & 0x100, 0x100);
    }

    /// **A unit's hit points are the `OBJECT` block's, at `OBJECT`'s own
    /// indent** (item 484).
    ///
    /// `myhits`, `damage` and `damage_frac` are written between the
    /// `SUBOBJECT` that closes above them and the `UNITDATA` fields that
    /// resume below, so there are two ways to read them off the wrong
    /// block and both read as silence: `SUBOBJECT`'s indent finds
    /// nothing, and `UNITDATA`'s finds nothing here and something else on
    /// a record that happens to write a like-named field. That is how
    /// `WallData::build_masks` was `None` on every capture ever taken
    /// (item 478) — a field the dump prints on every block, never parsed
    /// and therefore never compared.
    ///
    /// **Made to fail on purpose**: the fixture puts a decoy `damage`,
    /// `myhits` and `damage_frac` at `UNITDATA`'s own indent *and* inside
    /// `SUBOBJECT`, so a reader that goes out one level reads `7/700/70`
    /// and one that goes in reads `9/900/90`. Both redden on the value
    /// rather than on `None`, which a capture without a decoy could not
    /// do.
    #[test]
    fn the_unit_s_hit_points_are_the_object_block_s() {
        const TEXT: &str = "\
BEGIN GAME
 BEGIN WORLD
  seed 1
 BEGIN UNITDATA
  BEGIN OBJECT
   BEGIN SUBOBJECT
    flags 65
    o 3
    who 1
    x_internal 2424
    y_internal 7800
    z_internal 0
    damage 9
    myhits 900
    damage_frac 90
   damage 34
   uid 18
   myhits 120
   damage_frac 8
   mylos 4
  damage 7
  myhits 700
  damage_frac 70
  stance 1
  BEGIN GUY
   type 69
   x 2424
   y 7800
 BEGIN FRAME 1
";
        let log = Log::parse(TEXT);
        let init = log.initial().expect("the sample has a start block");
        assert_eq!(init.units.len(), 1);
        let u = &init.units[0];
        assert_eq!(
            (u.myhits, u.damage, u.damage_frac),
            (Some(120), Some(34), Some(8)),
            "a unit's hit points came off `UNITDATA` (7/700/70) or \
             `SUBOBJECT` (9/900/90) rather than `OBJECT`"
        );
        // The neighbours on the same block, so a reader that finds the
        // right block by accident on this fixture alone cannot pass.
        assert_eq!((u.uid, u.mylos), (Some(18), Some(4)));
        // And `UNITDATA`'s own field is still `UNITDATA`'s: the decoys
        // above it do not move the outer read.
        assert_eq!(u.stance, Some(1));
    }

    #[test]
    fn initial_state_is_typed() {
        let log = Log::parse(SAMPLE);
        let init = log.initial().unwrap();
        assert_eq!(
            init.game_info.iter().find(|(k, _)| *k == "(int)seed"),
            Some(&("(int)seed", "12345"))
        );
        assert_eq!(init.players.len(), 2);
        assert_eq!(
            init.world.iter().find(|(k, _)| *k == "xs"),
            Some(&("xs", "60"))
        );
        assert_eq!(
            init.cities,
            vec![CityDump {
                x: 3168,
                y: 30816,
                pop: 1,
                who: 0,
                // The sample's block carries the four fields the parser
                // used to read and nothing else, so every widened field is
                // its own default — `race` and the two identities at their
                // "no record" sentinel, the rest at zero.
                race: -1,
                o: -1,
                reg: -1,
                ..CityDump::default()
            }]
        );
        // One building, not two: the end-of-game dump's `2007` sits at the
        // same depth as the frames, after them, and is not the start state.
        assert_eq!(init.builds.len(), 1);
        assert_eq!(init.builds[0].o, 2000);
        assert_eq!(init.builds[0].flags, 39);
        assert!(init.units.iter().all(|u| u.o != 9));
        // `BUILDS=6`'s `orig_type` and `BUILDS=7`'s flat mining list.
        assert_eq!(init.builds[0].orig_type, Some(418));
        assert_eq!(
            init.builds[0].gather_from,
            vec![(20, 146), (21, 147), (22, 148)]
        );
        assert_eq!(
            init.builds[0].pos,
            Pos {
                x: 3168,
                y: 30816,
                z: 536
            }
        );
        assert_eq!(init.units.len(), 1);
        let u = &init.units[0];
        assert_eq!((u.flags, u.o, u.who), (65, 0, 0));
        assert_eq!(u.guys.len(), 2);
        assert_eq!(u.guys[0].kind, Some(69));
        assert_eq!(
            u.guys[1].pos,
            Some(Pos {
                x: 4254,
                y: 32555,
                z: 548
            })
        );
        assert_eq!(init.leaders.len(), 2);
        assert_eq!(init.leaders[0].tribe, 11);
        // The pair written *before* who-0's block, via `records`' zip — not
        // the 176160787 the raw block carries. Bit 2 is `is_human`.
        assert_eq!(init.leaders[0].leader_flags, 176160775);
        assert_eq!(init.leaders[1].leader_flags, 176160787);
    }

    #[test]
    fn frames_are_typed_and_per_frame_guys_are_empty() {
        let log = Log::parse(SAMPLE);
        let frames = log.frame_states();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].n, 1);
        assert_eq!(frames[1].n, 2);
        let u = &frames[0].units[0];
        assert_eq!(u.flags, 73);
        assert_eq!(u.guys.len(), 2);
        assert_eq!(u.guys[0], Guy::default());
        assert_eq!(frames[0].leaders[0].score, 181);
        assert_eq!(frames[0].leaders[0].leader_flags, 33554451);
        assert_eq!(frames[1].units[0].pos.x, 4250);
    }

    #[test]
    fn world_cells_split_the_flat_run_at_each_flags_line() {
        let text = "BEGIN GAME\n BEGIN WORLD\n  xs 2\n  ys 1\n  seed 5\n  GRASS\n  flags 256\n  goods 3\n  COAST\n  who -1\n  who2 -1\n  region 1\n  region2 63\n  val 40\n  land_sub 2\n  light 0\n  blocked 0\n  bad 0\n  solid 0\n  down -1\n  down_who -1\n  was_seen 0\n  WATER\n  flags 0\n  goods 0\n  none\n  who -1\n  who2 -1\n  region 63\n  region2 -1\n  val 0\n  land_sub 0\n";
        let log = Log::parse(text);
        let init = log.initial().unwrap();
        let cells = world_cells(&init.world);
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].land, "GRASS");
        assert_eq!(cells[0].flags, 256);
        assert_eq!(cells[0].goods, 3);
        assert_eq!(cells[0].flag_words, "COAST");
        assert_eq!(cells[0].region, 1);
        assert_eq!(cells[0].region2, 63);
        assert_eq!(cells[0].val, 40);
        assert_eq!(cells[0].down, -1);
        assert_eq!(cells[1].land, "WATER");
        assert_eq!(cells[1].flag_words, "none");
        assert_eq!(cells[1].region, 63);
        assert_eq!(cells[1].region2, -1);
    }

    #[test]
    fn an_empty_text_is_an_empty_log() {
        let log = Log::parse("");
        assert!(log.roots().is_empty());
        assert!(log.initial().is_none());
        assert!(log.frame_states().is_empty());
    }

    #[test]
    fn micro_reads_printed_floats_exactly() {
        assert_eq!(micro("369.375000"), Some(369_375_000));
        assert_eq!(micro("0.000000"), Some(0));
        assert_eq!(micro("-2.5"), Some(-2_500_000));
        assert_eq!(micro("12"), Some(12_000_000));
        assert_eq!(micro("1.2345678"), Some(1_234_567));
        assert_eq!(micro("x"), None);
        assert_eq!(micro(""), None);
    }

    /// The height grid as `dump_all` prints it: no block of its own, so it
    /// rides on the `UnbuiltForts` block that precedes it (whose own list
    /// is `length 0`).
    #[test]
    fn terrain_heights_ride_on_the_unbuilt_forts_block() {
        let text = "BEGIN DUMP\n BEGIN UnbuiltForts\n  length 0\n  size 4\n  increment -1\n  flags 0\n\
                    \n  length 1001\n  size 1001\n  increment -1\n  flags 0\n";
        let mut t = text.to_string();
        for i in 0..1001 {
            t.push_str(&format!("  list[scan] {}.375000\n", i));
        }
        // The waterline arrays follow at the same indent — more `list[scan]`
        // on the same block, which must not be taken for heights.
        t.push_str("  length 3\n  size 4\n  increment -1\n  flags 0\n");
        t.push_str("  list[scan] 36795\n  list[scan] 36554\n  list[scan] 36314\n");
        t.push_str(" BEGIN REGIONS\n  sea 70\n");
        let log = Log::parse(&t);
        let h = log.terrain_heights();
        assert_eq!(h.len(), 1001);
        assert_eq!(h[0], 375_000);
        assert_eq!(h[1000], 1_000_375_000);
        assert!(
            Log::parse("BEGIN GAME\n x 1\n")
                .terrain_heights()
                .is_empty()
        );
    }

    /// The trace's shape as run11 writes it (CRLF, the subsystem checksums
    /// between `LINE` and the seed, the seed printed signed) and the two
    /// ways a record can be incomplete.
    #[test]
    fn checksum_records_are_read_from_the_preamble() {
        let text = "CHECKSUM 0\r\n    FILE game.cpp\r\n    LINE 6136\r\nAmmo 1\r\nWorld 1\r\n\
                    Rules 2650943563\r\ngame_random seed 12345\r\n\
                    CHECKSUM 1\r\n    FILE leaders.cpp\r\n    LINE 13457\r\n\
                    game_random seed -232153382\r\n\
                    CHECKSUM 2\r\n    FILE setup.cpp\r\n    LINE 1244\r\nAmmo 1\r\n\
                    BEGIN GAME INFO\r\n WHO = 0\r\n";
        let log = Log::parse(text);
        let c = log.checksums();
        assert_eq!(
            c,
            vec![
                Checksum {
                    n: 0,
                    file: "game.cpp",
                    line: 6136,
                    seed: 12345
                },
                Checksum {
                    n: 1,
                    file: "leaders.cpp",
                    line: 13457,
                    seed: 0xf2299eda
                },
            ],
            "the third record has no seed line and is dropped"
        );
    }

    /// A whole `UNITS=3` unit, copied from run29's block 15100 — the AI's
    /// Cataphract `who 1 o 25`, three orders deep with a six-entry path
    /// stack. Trimmed only of the guy's animation fields.
    ///
    /// Everything the widening added is asserted here, and three traps of
    /// `docs/ORDERS.md` §11.1 with it: the class chain is the block
    /// *nesting*, so `ATTACKTOORDER`'s geometry sits on its `MOVEORDER`
    /// child and a bare `MOVEORDER` carries its own; the list is written
    /// **newest first**; and `ox/whom/uid` sit one level shallower than
    /// `flags`, which is what keeps them on `TARGETORDER`.
    #[test]
    fn a_units_3_record_is_read_whole_orders_included() {
        let text = "\
BEGIN GAME
 BEGIN FRAME 15100
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 1
     o 25
     who 1
     x_internal 36156
     y_internal 35196
     z_internal 102
    damage 7
    uid 86
    myhits 85
   angle -2147483648
   dest_angle -2147483648
   tolerance 0
   orders_x 34968
   orders_y 36696
   group 69
   myspeed 30
   unit_masks 0
   unit_masks2 0
   inside_up -1
   form 0
   form_mod 50
   path_recursion 0
   idle 0
   stance 0
   o_up -1
   BEGIN STACK<TYPE>
    size 20
    length 6
    increment 10
    BEGIN PATHDATA
     to_x 35592
     to_y 33480
     tolerance 0
     flags 1
    BEGIN PATHDATA
     to_x 36287
     to_y 35327
     tolerance 0
     flags 8
   length 3
   type 2
   metric 0
   BEGIN ATTACKTOORDER
    BEGIN MOVEORDER
     BEGIN UNITORDER
      flags 1
     x 35592
     y 33480
     angle 974716928
     dest 1
     tolerance 0
     pause 0
     retry 0
     attempts 0
     timer 0
     facing 1
     dest_x 35592
     dest_y 33288
     last_x -1
     last_y -1
     coll_x 0
     coll_y 0
     orig_x 35592
     orig_y 33480
     off_x 264
     off_y 456
   type 10
   metric 0
   BEGIN ATTACKORDER
    BEGIN TARGETORDER
     BEGIN UNITORDER
      flags 16
     ox 16
     whom 0
     uid 33
    mandatory 0
    defensive 0
    in_range 0
    ever_in_range 0
    new_ord 1
    def_x -1
    def_y -1
   type 1
   metric 0
   BEGIN MOVEORDER
    BEGIN UNITORDER
     flags 1
    x 34968
    y 36696
    angle -1966669824
    dest 0
    tolerance 0
    pause 0
    retry 0
    attempts 0
    timer 0
    facing -1
    dest_x 36156
    dest_y 35196
    last_x 36156
    last_y 34812
    coll_x 0
    coll_y 0
    orig_x -1
    orig_y -1
    off_x 408
    off_y 600
   BEGIN GUY
    type 227
    x 36156
    y 35196
    z 99
    angle -2147483648
";
        let log = Log::parse(text);
        let frames = log.frame_states();
        let u = &frames[0].units[0];
        assert_eq!((u.who, u.o, u.flags), (1, 25, 1));
        assert_eq!(u.guys[0].kind, Some(227), "the TypeIndex of a Cataphract");

        // The order layer's own `UNITDATA` fields.
        assert_eq!(u.group, Some(69));
        assert_eq!((u.form, u.form_mod), (Some(0), Some(50)));
        assert_eq!((u.orders_x, u.orders_y), (Some(34968), Some(36696)));
        assert_eq!(u.angle, Some(-2147483648));
        assert_eq!(u.dest_angle, Some(-2147483648));
        assert_eq!(u.stance, Some(0));
        assert_eq!(u.myspeed, Some(30));
        assert_eq!(u.o_up, Some(-1), "a captain");
        assert_eq!(u.inside_up, Some(-1));
        assert_eq!((u.myhits, u.damage), (Some(85), Some(7)));
        assert_eq!(
            (u.tolerance, u.path_recursion, u.idle),
            (Some(0), Some(0), Some(0))
        );

        // Three orders, newest first as the log writes them.
        let kinds: Vec<&str> = u.orders.iter().map(|o| o.kind.as_str()).collect();
        assert_eq!(kinds, ["ATTACKTOORDER", "ATTACKORDER", "MOVEORDER"]);
        assert_eq!(
            u.orders.iter().map(|o| o.index).collect::<Vec<_>>(),
            [2, 10, 1],
            "the `type` lines pair with the blocks by position"
        );
        let cur = u.current_order().expect("the order being executed");
        assert_eq!(cur.kind, "MOVEORDER");
        assert_eq!(
            u.orders_front_first().next().unwrap().kind,
            "MOVEORDER",
            "front-first is the log's list reversed"
        );

        // The whole `MOVEORDER` row of the order being executed.
        assert_eq!((cur.x, cur.y), (Some(34968), Some(36696)));
        assert_eq!(cur.angle, Some(-1966669824));
        assert_eq!(cur.dest, Some(0));
        assert_eq!(cur.facing, Some(-1));
        assert_eq!((cur.dest_x, cur.dest_y), (Some(36156), Some(35196)));
        assert_eq!((cur.last_x, cur.last_y), (Some(36156), Some(34812)));
        assert_eq!((cur.coll_x, cur.coll_y), (Some(0), Some(0)));
        assert_eq!((cur.orig_x, cur.orig_y), (Some(-1), Some(-1)));
        assert_eq!((cur.off_x, cur.off_y), (Some(408), Some(600)));
        assert_eq!(
            (cur.tolerance, cur.pause, cur.retry, cur.attempts, cur.timer),
            (Some(0), Some(0), Some(0), Some(0), Some(0))
        );

        // The `ATTACKTOORDER`'s geometry is on its `MOVEORDER` child, and
        // it is a different row from the bare move's.
        let to = &u.orders[0];
        assert_eq!((to.x, to.y), (Some(35592), Some(33480)));
        assert_eq!(to.facing, Some(1));
        assert_eq!((to.off_x, to.off_y), (Some(264), Some(456)));
        assert_eq!(to.flags, 1);

        // `ATTACKORDER`, which no dump had shown before run29: the
        // `TARGETORDER` base and the seven fields read back from the PE.
        let atk = &u.orders[1];
        assert_eq!((atk.ox, atk.whom, atk.uid), (Some(16), Some(0), Some(33)));
        assert_eq!(
            atk.flags, 0x10,
            "§1.3's bit 0x10 — `fight`'s re-target request — and not the \
             action bit"
        );
        assert!(!atk.is_action());
        assert_eq!(atk.mandatory, Some(0));
        assert_eq!(atk.defensive, Some(0));
        assert_eq!(atk.in_range, Some(0));
        assert_eq!(atk.ever_in_range, Some(0));
        assert_eq!(atk.new_ord, Some(1));
        assert_eq!((atk.def_x, atk.def_y), (Some(-1), Some(-1)));
        // And no move geometry leaks onto it from the neighbours.
        assert_eq!((atk.x, atk.y, atk.off_x), (None, None, None));

        // The path stack, bottom (the goal) first.
        assert_eq!(u.path.len(), 2);
        assert_eq!(u.path[0].to, (35592, 33480));
        assert_eq!(u.path[0].flags, 1, "the goal");
        assert_eq!(u.path[1].flags, 8);
    }

    /// `GroupMoveOrder`, the order a **human's** group move gives each
    /// member — run31's own text, and the record no dump had held.
    ///
    /// Two things about it are traps. Its block name is the only one in the
    /// family that is **not** upper case, so the order walk's
    /// `ends_with("ORDER")` dropped it *and* slid every later `type` onto
    /// the wrong body. And it carries **two** bases: `MOVEORDER` holds the
    /// member's own slot destination in `x`/`y` with the click in
    /// `orig_x`/`orig_y`, while `GROUPORDER` holds `oxx` — the object
    /// `GroupData::find_leader` chose — and `form_id`, the member's index
    /// into the group's parallel arrays.
    #[test]
    fn a_group_move_order_carries_both_bases_and_names_its_leader() {
        let text = "\
BEGIN GAME
 BEGIN FRAME 204
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 9
     o 9
     who 0
     x_internal 5928
     y_internal 6072
   group 1
   BEGIN STACK<TYPE>
    size 10
    length 1
    increment 10
    BEGIN PATHDATA
     to_x 9077
     to_y 5629
     tolerance 0
     flags 1
   length 1
   type 19
   metric 0
   BEGIN GroupMoveOrder
    BEGIN MOVEORDER
     BEGIN UNITORDER
      flags 5
     x 9096
     y 5640
     angle 927662080
     dest 0
     tolerance 0
     pause 0
     retry 0
     attempts 0
     timer 0
     facing 0
     dest_x 9096
     dest_y 5640
     last_x -1
     last_y -1
     coll_x 0
     coll_y 0
     orig_x 9123
     orig_y 5841
     off_x 648
     off_y 264
    BEGIN GROUPORDER
     BEGIN UNITORDER
      flags 5
     oxx 9
     whose 0
     group_angle 927662080
     id 203100
     form_id 15
    in_group 0
   length 1
   size 1
   increment 1
   BEGIN GUY
    type 132
";
        let log = Log::parse(text);
        let frames = log.frame_states();
        let u = &frames[0].units[0];
        assert_eq!((u.who, u.o, u.group), (0, 9, Some(1)));
        assert_eq!(u.orders.len(), 1, "the mixed-case block is not dropped");
        let o = &u.orders[0];
        assert_eq!(o.kind, "GroupMoveOrder");
        assert_eq!(o.index, 19, "and the positional type still pairs with it");
        // The `MoveOrder` base: this member's slot, and the click it came from.
        assert_eq!((o.x, o.y), (Some(9096), Some(5640)));
        assert_eq!((o.orig_x, o.orig_y), (Some(9123), Some(5841)));
        assert_eq!(o.angle, Some(927_662_080));
        assert_eq!(
            (o.off_x, o.off_y),
            (Some(648), Some(264)),
            "the cell offset"
        );
        assert_eq!(o.flags, 5);
        // The `GroupOrder` base, and `GroupMoveOrder`'s own last field.
        assert_eq!(o.oxx, Some(9), "the leader find_leader chose");
        assert_eq!(o.whose, Some(0));
        assert_eq!(o.group_angle, Some(927_662_080));
        assert_eq!(o.group_id, Some(203_100));
        assert_eq!(o.form_id, Some(15), "this member's slot in the group");
        assert_eq!(o.in_group, Some(0));
        // And nothing from the attack row leaks onto it.
        assert_eq!((o.mandatory, o.ox, o.whom), (None, None, None));
    }
}
