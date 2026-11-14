//! The executable's stores to a leader record's field, found by displacement
//! and told apart by the base register's provenance (item 4159's frozen scan,
//! `docs/DECISIONS.md` entry 131 §1).
//!
//! A leaf of the leaders checksum line that nothing in a capture can move
//! leaves `LEADERS_HELD_OUT` as **frozen** when every store the executable
//! makes to it writes one constant, the constant our walk holds. This module
//! is the census that claim stands on: for a field at `[lo, hi)` of
//! `LeaderData` it lists every instruction of `riseofnations.exe` that writes
//! those bytes through a pointer to a leader record, and every one it cannot
//! rule out.
//!
//! # Why displacement alone is not enough
//!
//! A displacement names a field only once the base is known to be a leader.
//! `+0x270` (`dow`) is written by 724 instructions of the listing, two of
//! them `Leader::action_declare`'s; `+0x1c` (`score_explored`) by 1,453. So
//! the scan runs a small forward dataflow over each function of the linker
//! map (`sbl/rise_z.map`), on `llvm-objdump`'s listing, and classifies every
//! general register as one of:
//!
//! - **a leader pointer at a known offset** (`V::Leader`): `this` on entry to
//!   a `Leader`, `LeaderData`, `LeaderOut` or `Leaders` method (each a prefix
//!   of `Leader`; `Leaders::list` is `Leader[10]` at `+0`), any address in
//!   `leaders.list` (`?leaders@@3VLeaders@@A`, the map's), a load of the two
//!   references to it (`GameAccess::leaders`, `GameAccessConst::leadersc`),
//!   and the return of a function whose decorated name returns one;
//! - **not a leader** (`V::Other`, `V::Stack`): a load from any memory but the
//!   stack — **no record type of `rise.pdb` has a field typed as a pointer to
//!   any of the four** (`types.txt`, 2026-11-13), so under the program's own
//!   types a pointer loaded from the heap or a global is not a leader's —
//!   the `this` of any other class's method, arithmetic results, constants
//!   outside `leaders.list`, and the stack frame;
//! - **what the caller left** (`V::Arg`): an argument, a register on entry,
//!   a load off the stack no tracked frame slot answers. It can be a leader
//!   only where a caller passed one (an escape names its callee), the
//!   function's decorated name mentions one, or the function spilled one
//!   off `esp` itself;
//! - **unknown** (`V::Top`): a merge of a leader with anything else, a
//!   `pop`, an undecorated callee's return (`_memset` returns its argument),
//!   and every instruction no path reaches.
//!
//! The `ebp` frame's slots are tracked, so a `this` spilled to `[ebp - k]`
//! and reloaded stays a leader. A pointer a loop walks by a stride holds a
//! span from where it starts, widened at the back edge to `SPAN_CAP` so
//! the fixpoint converges, and a store through it is charged to the element
//! the walk starts at.
//!
//! # What a field's scan reports
//!
//! - [`FieldScan::direct`]: stores through a leader pointer whose bytes meet
//!   `[lo, hi)`, each with the value stored when it is a constant.
//! - [`FieldScan::unresolved`]: stores at the displacement through an
//!   unknown base inside the functions a leader pointer can reach (leader
//!   methods, functions that ever hold one, functions whose decorated names
//!   mention one, and the callees a leader pointer escapes to), or through a
//!   caller's value inside the functions one can arrive in. Each is a site
//!   the scan cannot tell apart; the test that pins it says why it is not a
//!   leader's, and one no path reaches says so itself ([`Site::reached`]).
//! - [`Scan::escapes`]: every leader pointer pushed as an argument, passed
//!   as another class's `this`, run under a string store, or stored to the
//!   heap. A callee typed to its own record writes inside that record, so
//!   under the program's types only an untyped callee (`_memset`, a data
//!   walk, an indirect call) can reach a scalar field beyond the offset it
//!   was handed; a field's verdict names those.
//!
//! # What it does not see
//!
//! A leader pointer laundered through an integer or a `void *` field and
//! loaded back elsewhere; a write by a function the map does not name; a
//! spill off `esp` (it widens the function's scope instead of being
//! tracked); a loop that walks a record past the array it starts in, or
//! past `SPAN_CAP`; code reached only through an indirect jump the pass
//! cannot read a table for; and what an untyped callee writes, which the
//! verdict judges by the callee (a whole-record clear, a save game's walk).
//! The scan is a census under the program's types, not a proof.

use std::collections::{BTreeMap, BTreeSet};

/// `sizeof(Leader)` (`rise.pdb`).
pub const LEADER_SIZE: u32 = 0x6eec;
/// `Leaders::list` is `Leader[10]` at `+0` (`rise.pdb`).
pub const LEADERS_LIST_LEN: u32 = 10;

/// The classes whose `this` is a leader record at `+0`.
const LEADER_CLASSES: &[&str] = &["Leader", "LeaderData", "LeaderOut", "Leaders"];

/// Runtime helpers that keep `ecx` (the `this` they are called beside):
/// the frame prologues and the stack probe.
const HELPERS: &[&str] = &[
    "__EH_prolog",
    "__SEH_prolog4",
    "__alloca_probe",
    "__alloca_probe_8",
    "__alloca_probe_16",
    "__chkstk",
];

/// One function of the linker map: its start and every name folded onto it.
#[derive(Clone, Debug)]
pub struct Func {
    pub start: u32,
    pub names: Vec<String>,
}

/// The parts of `rise_z.map` the scan reads.
#[derive(Debug, Default)]
pub struct Map {
    /// Code symbols marked `f`, by address, folded names together.
    pub funcs: Vec<Func>,
    /// Every other symbol's address, by decorated name.
    pub data: BTreeMap<String, u32>,
}

impl Map {
    /// Parses the map's `Publics by Value` and `Static symbols` tables.
    pub fn parse(text: &str) -> Map {
        let mut funcs: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        let mut data = BTreeMap::new();
        for line in text.lines() {
            let mut it = line.split_whitespace();
            let (Some(sect), Some(name), Some(va)) = (it.next(), it.next(), it.next()) else {
                continue;
            };
            let Some((s, o)) = sect.split_once(':') else {
                continue;
            };
            if s.len() != 4 || o.len() != 8 || u32::from_str_radix(o, 16).is_err() {
                continue;
            }
            let Ok(va) = u32::from_str_radix(va, 16) else {
                continue;
            };
            if va == 0 {
                continue;
            }
            if s == "0001" && it.next() == Some("f") {
                let names = funcs.entry(va).or_default();
                if !names.iter().any(|n| n == name) {
                    names.push(name.to_owned());
                }
            } else {
                data.entry(name.to_owned()).or_insert(va);
            }
        }
        Map {
            funcs: funcs
                .into_iter()
                .map(|(start, names)| Func { start, names })
                .collect(),
            data,
        }
    }
}

/// The class a decorated member name qualifies, if it is one:
/// `?compute_score@Leader@@QAEXH@Z` → `Leader`, `??0Leaders@@QAE@XZ` →
/// `Leaders`.
pub fn class_of(name: &str) -> Option<&str> {
    let rest = if let Some(r) = name.strip_prefix("??") {
        // A special name: `??0` constructor, `??1` destructor, `??_G` and
        // the other two-character codes.
        let skip = if r.starts_with('_') { 2 } else { 1 };
        r.get(skip..)?
    } else {
        let r = name.strip_prefix('?')?;
        &r[r.find('@')? + 1..]
    };
    let end = rest.find("@@")?;
    let class = &rest[..end];
    // A nested qualifier `Inner@Outer` names the innermost class first.
    let class = class.split('@').next()?;
    (!class.is_empty()).then_some(class)
}

fn is_leader_class(name: &str) -> bool {
    class_of(name).is_some_and(|c| LEADER_CLASSES.contains(&c))
}

/// Whether a member is `__thiscall` (`E` after the access and const codes).
fn is_thiscall(name: &str) -> bool {
    let Some(i) = name.find("@@") else {
        return false;
    };
    let code = name.as_bytes().get(i + 2..i + 5).unwrap_or(&[]);
    // The member access codes (private, protected, public; plain, virtual
    // and their far forms), a const qualifier, then `E`.
    matches!(
        code,
        [
            b'A' | b'B' | b'E' | b'F' | b'I' | b'J' | b'M' | b'N' | b'Q' | b'R' | b'U' | b'V',
            b'A'..=b'D',
            b'E'
        ]
    )
}

/// Whether a decorated name returns a pointer or reference to a leader.
fn returns_leader(name: &str) -> bool {
    let Some(i) = name.find("@@") else {
        return false;
    };
    let tail = &name[i + 2..];
    // Access, optional const, calling convention, then the return type.
    for skip in [2usize, 3] {
        if let Some(t) = tail.get(skip..) {
            for p in ["PAV", "PBV", "AAV", "ABV"] {
                if let Some(t) = t.strip_prefix(p)
                    && LEADER_CLASSES
                        .iter()
                        .any(|c| t.strip_prefix(c).is_some_and(|r| r.starts_with("@@")))
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether a decorated name mentions a leader type anywhere.
fn mentions_leader(name: &str) -> bool {
    LEADER_CLASSES
        .iter()
        .any(|c| name.contains(&format!("V{c}@@")))
}

// ---------------------------------------------------------------------------
// The listing.

const R32: [&str; 8] = ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
const ESP: usize = 4;
const EBP: usize = 5;
const EAX: usize = 0;
const ECX: usize = 1;
const EDX: usize = 2;
const ESI: usize = 6;
const EDI: usize = 7;

fn reg32(s: &str) -> Option<usize> {
    R32.iter().position(|r| *r == s)
}

/// A sub-register's containing register, with its width in bytes.
fn subreg(s: &str) -> Option<(usize, u8)> {
    Some(match s {
        "al" | "ah" => (EAX, 1),
        "cl" | "ch" => (ECX, 1),
        "dl" | "dh" => (EDX, 1),
        "bl" | "bh" => (3, 1),
        "ax" => (EAX, 2),
        "cx" => (ECX, 2),
        "dx" => (EDX, 2),
        "bx" => (3, 2),
        "sp" => (ESP, 2),
        "bp" => (EBP, 2),
        "si" => (ESI, 2),
        "di" => (EDI, 2),
        _ => return None,
    })
}

/// A memory operand: `[base + scale*index + disp]`, with its access width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mem {
    pub base: Option<usize>,
    pub index: Option<(usize, u8)>,
    pub disp: u32,
    /// A segment override (`fs:` is the thread block, never a record).
    pub seg: bool,
    pub width: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opnd {
    Reg(usize),
    Sub(usize, u8),
    Imm(u32),
    Mem(Mem),
    Other,
}

fn parse_imm(s: &str) -> Option<u32> {
    let (neg, t) = match s.strip_prefix('-') {
        Some(t) => (true, t),
        None => (false, s),
    };
    let v = u32::from_str_radix(t.strip_prefix("0x")?, 16).ok()?;
    Some(if neg { v.wrapping_neg() } else { v })
}

fn parse_mem(s: &str) -> Option<Mem> {
    let open = s.find('[')?;
    let close = s[open..].find(']')? + open;
    let head = &s[..open];
    let width = if head.starts_with("byte") {
        1
    } else if head.starts_with("word") {
        2
    } else if head.starts_with("dword") {
        4
    } else if head.starts_with("qword") {
        8
    } else if head.starts_with("tbyte") {
        10
    } else if head.starts_with("xmmword") {
        16
    } else {
        4
    };
    let seg = head.contains(':') && !head.contains("es:");
    let mut m = Mem {
        base: None,
        index: None,
        disp: 0,
        seg,
        width,
    };
    let inner = &s[open + 1..close];
    let mut sign = 1i64;
    for tok in inner.split_whitespace() {
        match tok {
            "+" => sign = 1,
            "-" => sign = -1,
            t => {
                if let Some((sc, r)) = t.split_once('*') {
                    m.index = Some((reg32(r)?, sc.parse().ok()?));
                } else if let Some(r) = reg32(t) {
                    if m.base.is_none() {
                        m.base = Some(r);
                    } else {
                        m.index = Some((r, 1));
                    }
                } else if let Some(v) = parse_imm(t) {
                    let v = if sign < 0 { v.wrapping_neg() } else { v };
                    m.disp = m.disp.wrapping_add(v);
                } else if t == "eiz" {
                } else {
                    return None;
                }
                sign = 1;
            }
        }
    }
    Some(m)
}

fn parse_opnd(s: &str) -> Opnd {
    let s = s.trim();
    if let Some(r) = reg32(s) {
        Opnd::Reg(r)
    } else if let Some((r, w)) = subreg(s) {
        Opnd::Sub(r, w)
    } else if let Some(v) = parse_imm(s) {
        Opnd::Imm(v)
    } else if s.contains('[') {
        parse_mem(s).map_or(Opnd::Other, Opnd::Mem)
    } else {
        Opnd::Other
    }
}

/// One instruction of the listing.
#[derive(Clone, Debug)]
pub struct Insn {
    pub at: u32,
    /// The mnemonic, with a `rep`/`lock` prefix folded into it.
    pub op: String,
    pub args: String,
}

impl Insn {
    /// Parses one `llvm-objdump --no-show-raw-insn -M intel` line.
    pub fn parse(line: &str) -> Option<Insn> {
        let (addr, rest) = line.split_once(':')?;
        let at = u32::from_str_radix(addr.trim(), 16).ok()?;
        let rest = rest.trim();
        let (op, args) = rest.split_once('\t').unwrap_or((rest, ""));
        let (op, args) = (op.trim(), args.trim());
        if op.is_empty() {
            return None;
        }
        if matches!(op, "rep" | "repne" | "repe" | "lock") {
            let (op2, args2) = args.split_once('\t').unwrap_or((args, ""));
            let op2 = op2.trim();
            let folded = if op == "lock" {
                op2.to_owned()
            } else {
                format!("{op} {op2}")
            };
            return Some(Insn {
                at,
                op: folded,
                args: args2.trim().to_owned(),
            });
        }
        Some(Insn {
            at,
            op: op.to_owned(),
            args: args.to_owned(),
        })
    }

    fn operands(&self) -> Vec<Opnd> {
        if self.args.is_empty() {
            return Vec::new();
        }
        // Split on commas outside brackets.
        let mut out = Vec::new();
        let (mut depth, mut start) = (0i32, 0usize);
        for (i, c) in self.args.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(parse_opnd(&self.args[start..i]));
                    start = i + 1;
                }
                _ => {}
            }
        }
        // A branch's target carries a `<.text+…>` note after the address.
        let last = self.args[start..].split(" <").next().unwrap_or("");
        out.push(parse_opnd(last));
        out
    }

    /// A direct branch or call target.
    fn target(&self) -> Option<u32> {
        let a = self.args.split_whitespace().next()?;
        u32::from_str_radix(a.strip_prefix("0x")?, 16).ok()
    }
}

/// Whether `op` writes its first operand when that operand is memory.
fn writes_first(op: &str) -> bool {
    matches!(
        op,
        "mov"
            | "add"
            | "sub"
            | "and"
            | "or"
            | "xor"
            | "adc"
            | "sbb"
            | "inc"
            | "dec"
            | "neg"
            | "not"
            | "shl"
            | "sal"
            | "shr"
            | "sar"
            | "rol"
            | "ror"
            | "rcl"
            | "rcr"
            | "xchg"
            | "pop"
            | "cmpxchg"
            | "cmpxchg8b"
            | "xadd"
            | "bts"
            | "btr"
            | "btc"
            | "shld"
            | "shrd"
            | "movss"
            | "movsd"
            | "movq"
            | "movd"
            | "movups"
            | "movaps"
            | "movupd"
            | "movapd"
            | "movdqa"
            | "movdqu"
            | "movlps"
            | "movhps"
            | "movlpd"
            | "movhpd"
            | "movnti"
            | "movntps"
            | "movntdq"
            | "pextrw"
            | "fst"
            | "fstp"
            | "fist"
            | "fistp"
            | "fisttp"
            | "fbstp"
            | "fnstsw"
            | "fnstcw"
            | "fstsw"
            | "fstcw"
            | "fnsave"
            | "fnstenv"
    ) || op.starts_with("set")
}

/// Instructions whose first register operand is read, not written.
fn reads_first(op: &str) -> bool {
    matches!(
        op,
        "cmp"
            | "test"
            | "push"
            | "bt"
            | "jmp"
            | "call"
            | "ret"
            | "comiss"
            | "ucomiss"
            | "comisd"
            | "ucomisd"
            | "out"
            | "outsb"
            | "outsd"
    ) || op.starts_with('j')
}

fn string_op(op: &str, args: &str) -> bool {
    let base = op.rsplit(' ').next().unwrap_or(op);
    matches!(
        base,
        "stosb"
            | "stosw"
            | "stosd"
            | "movsb"
            | "movsw"
            | "lodsb"
            | "lodsw"
            | "lodsd"
            | "scasb"
            | "scasw"
            | "scasd"
            | "cmpsb"
            | "cmpsw"
    ) || (matches!(base, "movsd" | "cmpsd") && args.contains("es:[edi]"))
}

fn is_string_store(op: &str, args: &str) -> bool {
    let base = op
        .trim_start_matches("rep ")
        .trim_start_matches("repne ")
        .trim_start_matches("repe ");
    matches!(base, "stosd" | "stosb" | "stosw" | "movsb" | "movsw")
        || (base == "movsd" && args.contains("es:[edi]"))
}

// ---------------------------------------------------------------------------
// The value domain.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum V {
    /// Unknown: a merge of a leader with anything else, a `pop`, an
    /// undecorated callee's return, an instruction no path reaches.
    Top,
    /// What the caller left: an argument, a register on entry, a load off
    /// the stack that no tracked frame slot answers. A leader only where a
    /// caller passed one (an escape names its callee), the decorated name
    /// mentions one, or the function itself spilled one off `esp`.
    Arg,
    /// The stack frame.
    Stack,
    /// Known not to be a leader pointer.
    Other,
    Const(u32),
    /// `leaders.list[k] + o` for some slot `k` and some `o` in
    /// `[off, off + span]`; `exact` when no unknown index has been added to
    /// it. A span comes from a loop that walks the record by a stride, and
    /// is capped at `SPAN_CAP`.
    Leader {
        off: u32,
        span: u32,
        exact: bool,
    },
}

/// The widest union of two paths' leader offsets kept as one span.
const MERGE_CAP: u32 = 0x800;

/// A loop's widening: where a back edge's state carries a leader pointer
/// whose span grew from the loop head's, the span is set to `SPAN_CAP`
/// from the same start, so a walk by a constant stride converges in two
/// rounds instead of `SPAN_CAP / stride`.
fn widen(old: &Regs, mut next: Regs) -> Regs {
    fn one(old: V, new: V) -> V {
        match (old, new) {
            (
                V::Leader {
                    off: o1, span: s1, ..
                },
                V::Leader {
                    off: o2,
                    span: s2,
                    exact,
                },
            ) if o1 == o2 && s2 > s1 => V::Leader {
                off: o2,
                span: SPAN_CAP.max(s1),
                exact,
            },
            _ => new,
        }
    }
    for i in 0..8 {
        next.r[i] = one(old.r[i], next.r[i]);
    }
    let slots: Vec<(u32, V)> = next
        .slots
        .iter()
        .map(|&(k, v)| (k, old.slot(k).map_or(v, |o| one(o, v))))
        .collect();
    next.slots = slots;
    next
}

/// The fixpoint's rounds before a function is reported unconverged.
const ROUNDS: usize = 40;

/// The widest span a loop's walk of a record is widened to: 64 words, the
/// longest per-slot array below `+0x6d8` (`reg_known_rares`).
const SPAN_CAP: u32 = 0x100;

fn leader(off: u32, exact: bool) -> V {
    V::Leader {
        off: off % LEADER_SIZE,
        span: 0,
        exact,
    }
}

#[derive(Clone, Copy)]
struct Geometry {
    base: u32,
    end: u32,
}

impl Geometry {
    fn norm(&self, c: u32) -> V {
        if (self.base..self.end).contains(&c) {
            leader(c - self.base, true)
        } else {
            V::Const(c)
        }
    }
}

fn merge(a: V, b: V) -> V {
    use V::*;
    match (a, b) {
        _ if a == b => a,
        (
            Leader {
                off: x,
                span: sx,
                exact: e,
            },
            Leader {
                off: y,
                span: sy,
                exact: f,
            },
        ) => {
            let lo = x.min(y);
            let hi = (x + sx).max(y + sy);
            if hi - lo > MERGE_CAP {
                // Two unrelated places in the record: no one field.
                Top
            } else {
                Leader {
                    off: lo,
                    span: hi - lo,
                    exact: e && f && hi == lo,
                }
            }
        }
        (Leader { off, span, .. }, Const(0)) | (Const(0), Leader { off, span, .. }) => Leader {
            off,
            span,
            exact: false,
        },
        (Other | Const(_), Other | Const(_)) => Other,
        (Arg, Arg | Other | Const(_)) | (Other | Const(_), Arg) => Arg,
        _ => Top,
    }
}

/// The general registers, and the `ebp` frame's slots that hold a leader
/// pointer or an unknown (`[ebp + k]` while `ebp` is the frame). A frame slot
/// the map does not hold is a local that holds neither (below the frame) or
/// an argument (above it), which [`load`] tells apart.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Regs {
    r: [V; 8],
    slots: Vec<(u32, V)>,
}

impl std::ops::Index<usize> for Regs {
    type Output = V;
    fn index(&self, i: usize) -> &V {
        &self.r[i]
    }
}

impl std::ops::IndexMut<usize> for Regs {
    fn index_mut(&mut self, i: usize) -> &mut V {
        &mut self.r[i]
    }
}

impl Regs {
    fn swap(&mut self, a: usize, b: usize) {
        self.r.swap(a, b);
    }

    fn iter(&self) -> impl Iterator<Item = &V> {
        self.r.iter().chain(self.slots.iter().map(|(_, v)| v))
    }

    fn slot(&self, k: u32) -> Option<V> {
        self.slots.iter().find(|s| s.0 == k).map(|s| s.1)
    }

    fn set_slot(&mut self, k: u32, v: V) {
        self.slots.retain(|s| s.0 != k);
        if matches!(v, V::Leader { .. } | V::Top) {
            self.slots.push((k, v));
            self.slots.sort_unstable_by_key(|s| s.0);
        }
    }
}

/// A frame slot no store on a path has filled holds no leader there.
fn slot_or_absent(r: &Regs, k: u32) -> V {
    r.slot(k).unwrap_or(V::Other)
}

fn merge_regs(a: &Regs, b: &Regs) -> Regs {
    let mut out = a.clone();
    for (o, x) in out.r.iter_mut().zip(&b.r) {
        *o = merge(*o, *x);
    }
    let keys: BTreeSet<u32> = a.slots.iter().chain(&b.slots).map(|s| s.0).collect();
    out.slots.clear();
    for k in keys {
        let v = merge(slot_or_absent(a, k), slot_or_absent(b, k));
        if matches!(v, V::Leader { .. } | V::Top) {
            out.slots.push((k, v));
        }
    }
    out
}

fn unknown_regs() -> Regs {
    let mut r = [V::Top; 8];
    r[ESP] = V::Stack;
    Regs {
        r,
        slots: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// What the scan finds.

/// The value a store writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stored {
    Const(u32),
    /// Not a constant the pass can see: an increment, a register it does not
    /// hold a constant for, a copy.
    Live,
}

/// One store the scan reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    pub at: u32,
    /// The start of the function that holds it.
    pub func: u32,
    pub op: String,
    pub args: String,
    /// The offset in the record the store's first byte lands on (for a
    /// `direct` store) or its displacement (for an `unresolved` one).
    pub off: u32,
    pub width: u32,
    /// Whether an unknown index was added to the base.
    pub exact: bool,
    pub value: Stored,
    /// Whether any path the pass follows reaches it: a store no path
    /// reaches is in bytes no branch, fall-through or jump table leads to
    /// (a table's data decoded as code, or code entered some other way).
    pub reached: bool,
    /// For an unresolved store: its base is what the caller left (`V::Arg`).
    pub arg: bool,
}

/// How a leader pointer leaves the function that holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EscapeKind {
    /// Pushed before a call: an argument.
    Push,
    /// In `ecx` at a call of another class's `__thiscall` method.
    This,
    /// In `edi` under a string store (`rep stosd`, `movsd`).
    String,
    /// Stored to memory that is not the stack frame.
    Heap,
}

/// One leader pointer leaving its function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Escape {
    pub at: u32,
    pub func: u32,
    pub kind: EscapeKind,
    pub off: u32,
    pub exact: bool,
    /// The direct callee, when there is one.
    pub callee: Option<u32>,
    /// The instruction it leaves by: the call a push feeds, else `at`.
    pub via: u32,
}

/// A store to a displacement through an unknown base, before scoping.
#[derive(Clone, Debug)]
struct Raw {
    site: Site,
}

/// A field to scan: `[lo, hi)` of `LeaderData`.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub lo: u32,
    pub hi: u32,
}

/// One field's census.
#[derive(Clone, Debug, Default)]
pub struct FieldScan {
    pub direct: Vec<Site>,
    pub unresolved: Vec<Site>,
}

/// The whole scan: per field, and the program's escapes.
#[derive(Debug, Default)]
pub struct Scan {
    pub fields: BTreeMap<&'static str, FieldScan>,
    pub escapes: Vec<Escape>,
    /// Functions whose state did not converge (reported, never silent).
    pub unconverged: Vec<u32>,
    /// Functions the map names, and the instructions read.
    pub functions: usize,
    pub instructions: usize,
    /// The names folded onto each function start the scan reports.
    pub names: BTreeMap<u32, Vec<String>>,
}

impl Scan {
    /// The first name the map gives the function at `start`.
    pub fn name(&self, start: u32) -> &str {
        self.names
            .get(&start)
            .and_then(|n| n.first())
            .map_or("?", String::as_str)
    }
}

/// What the analysis is told about the image beyond the listing.
pub struct Context<'a> {
    pub map: &'a Map,
    /// `leaders` (`?leaders@@3VLeaders@@A`).
    pub leaders: u32,
    /// Static words that hold `leaders`' address: the two references.
    pub refs: BTreeSet<u32>,
    /// Reads a jump table's words out of the image.
    pub words: &'a dyn Fn(u32, usize) -> Option<Vec<u32>>,
    /// Seed `this` for leader-class methods (off only to perturb the scan).
    pub seed_this: bool,
}

struct FuncInfo {
    leader_class: bool,
    thiscall: bool,
    returns_leader: bool,
    helper: bool,
    /// `__EH_prolog` and `__SEH_prolog4` set `ebp` to the new frame.
    prolog: bool,
    mentions: bool,
    /// A C++ name (`?…`), whose return type the decoration states.
    decorated: bool,
}

fn info(names: &[String]) -> FuncInfo {
    FuncInfo {
        leader_class: names.iter().any(|n| is_leader_class(n)),
        thiscall: names.iter().any(|n| is_thiscall(n)),
        returns_leader: names.iter().any(|n| returns_leader(n)),
        helper: names.iter().any(|n| HELPERS.contains(&n.as_str())),
        prolog: names
            .iter()
            .any(|n| n == "__EH_prolog" || n == "__SEH_prolog4"),
        mentions: names.iter().any(|n| mentions_leader(n)),
        decorated: names.iter().any(|n| n.starts_with('?')),
    }
}

/// Runs the scan over a listing, one function at a time.
pub fn scan(ctx: &Context, listing: impl Iterator<Item = Insn>, fields: &[Field]) -> Scan {
    let geo = Geometry {
        base: ctx.leaders,
        end: ctx.leaders + LEADERS_LIST_LEN * LEADER_SIZE,
    };
    let starts: Vec<u32> = ctx.map.funcs.iter().map(|f| f.start).collect();
    let infos: BTreeMap<u32, FuncInfo> = ctx
        .map
        .funcs
        .iter()
        .map(|f| (f.start, info(&f.names)))
        .collect();

    let mut out = Scan::default();
    for f in fields {
        out.fields.insert(f.name, FieldScan::default());
    }
    let mut raw: Vec<(u32, Raw)> = Vec::new();
    // The functions an unknown base (`[0]`) and a caller's value (`[1]`) can
    // be a leader in.
    let mut scope: [BTreeSet<u32>; 2] = Default::default();
    let mut cur: Vec<Insn> = Vec::new();
    let mut cur_start: Option<u32> = None;
    let mut next_idx = 0usize;

    let flush = |start: Option<u32>,
                 body: &mut Vec<Insn>,
                 out: &mut Scan,
                 raw: &mut Vec<(u32, Raw)>,
                 scope: &mut [BTreeSet<u32>; 2]| {
        if let Some(s) = start
            && !body.is_empty()
        {
            let r = analyse(ctx, &geo, &infos, s, body, fields);
            out.functions += 1;
            out.instructions += body.len();
            if !r.converged {
                out.unconverged.push(s);
            }
            let fi = &infos[&s];
            if r.holds_leader || fi.leader_class || fi.mentions {
                scope[0].insert(s);
            }
            if fi.mentions || r.spills {
                scope[1].insert(s);
            }
            for (name, site) in r.direct {
                out.fields.get_mut(name).expect("field").direct.push(site);
            }
            for (_, site) in r.raw {
                raw.push((s, Raw { site }));
            }
            out.escapes.extend(r.escapes);
        }
        body.clear();
    };

    for insn in listing {
        while next_idx < starts.len() && starts[next_idx] <= insn.at {
            flush(cur_start, &mut cur, &mut out, &mut raw, &mut scope);
            cur_start = Some(starts[next_idx]);
            next_idx += 1;
        }
        if cur_start.is_some() {
            cur.push(insn);
        }
    }
    flush(cur_start, &mut cur, &mut out, &mut raw, &mut scope);

    for e in &out.escapes {
        if let Some(c) = e.callee {
            scope[0].insert(c);
            scope[1].insert(c);
        }
    }
    for (func, r) in raw {
        if !scope[usize::from(r.site.arg)].contains(&func) {
            continue;
        }
        for f in fields {
            if overlaps(r.site.off, r.site.width, f) {
                out.fields
                    .get_mut(f.name)
                    .expect("field")
                    .unresolved
                    .push(r.site.clone());
            }
        }
    }
    for f in &ctx.map.funcs {
        let reported = out.fields.values().any(|fs| {
            fs.direct
                .iter()
                .chain(&fs.unresolved)
                .any(|s| s.func == f.start)
        }) || out
            .escapes
            .iter()
            .any(|e| e.func == f.start || e.callee == Some(f.start));
        if reported {
            out.names.insert(f.start, f.names.clone());
        }
    }
    out
}

fn overlaps(off: u32, width: u32, f: &Field) -> bool {
    off < f.hi && off.saturating_add(width) > f.lo
}

struct Analysis {
    direct: Vec<(&'static str, Site)>,
    raw: Vec<(u32, Site)>,
    escapes: Vec<Escape>,
    holds_leader: bool,
    converged: bool,
    /// Whether the instruction being stepped is reached by no path.
    unreached: bool,
    /// Whether a leader or an unknown is ever stored to the stack off a
    /// frame slot the pass tracks (`[esp + k]`, or a frame it does not).
    spills: bool,
    /// Whether the instruction being stepped is in the prologue: the
    /// function's leading pushes and frame setup, whose pushes save
    /// registers or reserve a slot (`push ecx`) and pass nothing.
    prologue: bool,
}

fn analyse(
    ctx: &Context,
    geo: &Geometry,
    infos: &BTreeMap<u32, FuncInfo>,
    start: u32,
    body: &[Insn],
    fields: &[Field],
) -> Analysis {
    let end = body.last().map_or(start, |i| i.at + 1);
    let me = &infos[&start];
    let ops: Vec<Vec<Opnd>> = body.iter().map(Insn::operands).collect();

    // Branch targets inside the function, and jump tables' targets.
    let mut table_targets: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (i, insn) in body.iter().enumerate() {
        if insn.op == "jmp"
            && let [Opnd::Mem(m)] = ops[i].as_slice()
            && m.base.is_none()
            && m.index.is_some_and(|(_, s)| s == 4)
        {
            let mut ts = Vec::new();
            if let Some(words) = (ctx.words)(m.disp, 512) {
                for w in words {
                    if !(start..end).contains(&w) {
                        break;
                    }
                    ts.push(w);
                }
            }
            table_targets.insert(insn.at, ts);
        }
    }

    let mut entry = unknown_regs();
    for (i, v) in entry.r.iter_mut().enumerate() {
        if i != ESP {
            *v = V::Arg;
        }
    }
    if me.thiscall {
        entry[ECX] = if me.leader_class && ctx.seed_this {
            leader(0, true)
        } else if me.leader_class {
            V::Arg
        } else {
            V::Other
        };
    }

    // `STORES_TRACE=<hex start>` prints one function's states, every round.
    let trace = std::env::var("STORES_TRACE")
        .ok()
        .and_then(|t| u32::from_str_radix(t.trim_start_matches("0x"), 16).ok());
    let mut pending: BTreeMap<u32, Regs> = BTreeMap::new();
    pending.insert(start, entry);
    let mut converged = false;
    let mut result = None;
    for round in 0..ROUNDS {
        let last = round == ROUNDS - 1;
        let mut changed = false;
        let mut a = Analysis {
            direct: Vec::new(),
            raw: Vec::new(),
            escapes: Vec::new(),
            holds_leader: false,
            converged: false,
            unreached: false,
            spills: false,
            prologue: true,
        };
        let mut st: Option<Regs> = None;
        let mut pushes: Vec<(u32, V)> = Vec::new();
        for (i, insn) in body.iter().enumerate() {
            let p = pending.get(&insn.at);
            a.unreached = st.is_none() && p.is_none();
            let mut regs = match (st.take(), p) {
                (Some(s), Some(p)) => merge_regs(&s, p),
                (Some(s), None) => s,
                (None, Some(p)) => p.clone(),
                (None, None) => unknown_regs(),
            };
            if regs.iter().any(|v| matches!(v, V::Leader { .. })) {
                a.holds_leader = true;
            }
            if trace == Some(start) {
                eprintln!(
                    "T{round} {:x} {:?} | {} {}",
                    insn.at, regs, insn.op, insn.args
                );
            }
            let frame_setup = insn.op == "push"
                || (insn.op == "mov" && insn.args == "ebp, esp")
                || (matches!(insn.op.as_str(), "sub" | "and") && insn.args.starts_with("esp,"));
            if a.prologue && !frame_setup {
                // The prologue's pushes were arguments only if it ends in
                // the call they feed (`Leaders::Leaders` opens on one).
                a.prologue = false;
                if insn.op != "call" {
                    pushes.clear();
                }
            }
            step(
                ctx,
                geo,
                infos,
                me,
                start,
                insn,
                &ops[i],
                &mut regs,
                &mut pushes,
                &mut a,
                fields,
            );
            if a.unreached {
                // Bytes no path reaches (a jump table's data decoded as code)
                // send no state on: a branch there is not a branch.
                pushes.clear();
                continue;
            }
            // Successors.
            let mut live = true;
            let at = insn.at;
            let mut send = |t: u32, regs: &Regs, pending: &mut BTreeMap<u32, Regs>| {
                if !(start..end).contains(&t) {
                    return;
                }
                let next = match pending.get(&t) {
                    Some(old) if t <= at => widen(old, merge_regs(old, regs)),
                    Some(old) => merge_regs(old, regs),
                    None => regs.clone(),
                };
                if pending.get(&t) != Some(&next) {
                    pending.insert(t, next);
                    changed = true;
                }
            };
            if insn.op.starts_with('j') || insn.op.starts_with("loop") {
                if let Some(t) = insn.target() {
                    send(t, &regs, &mut pending);
                }
                if let Some(ts) = table_targets.get(&insn.at) {
                    for &t in ts {
                        send(t, &regs, &mut pending);
                    }
                }
                if insn.op == "jmp" {
                    live = false;
                }
            } else if matches!(
                insn.op.as_str(),
                "ret" | "int3" | "hlt" | "ud2" | "retf" | "iretd"
            ) {
                live = false;
            }
            if !live {
                pushes.clear();
            }
            st = live.then_some(regs);
        }
        if !changed || last {
            converged = !changed;
            a.converged = converged;
            result = Some(a);
            break;
        }
    }
    let mut a = result.expect("a round ran");
    a.converged = converged;
    a
}

#[allow(clippy::too_many_arguments)]
fn step(
    ctx: &Context,
    geo: &Geometry,
    infos: &BTreeMap<u32, FuncInfo>,
    me: &FuncInfo,
    start: u32,
    insn: &Insn,
    ops: &[Opnd],
    regs: &mut Regs,
    pushes: &mut Vec<(u32, V)>,
    a: &mut Analysis,
    fields: &[Field],
) {
    let _ = me;
    let op = insn.op.as_str();
    let val = |o: &Opnd, regs: &Regs| -> V {
        match *o {
            Opnd::Reg(r) => regs[r],
            Opnd::Sub(..) => V::Other,
            Opnd::Imm(v) => geo.norm(v),
            Opnd::Mem(m) => load(ctx, regs, &m),
            Opnd::Other => V::Other,
        }
    };

    // Stores.
    let first_mem = match ops.first() {
        Some(Opnd::Mem(m)) => Some(*m),
        _ => None,
    };
    let mut store_at: Option<Mem> = None;
    if writes_first(op) && !(op == "movsd" && insn.args.contains("es:[edi]")) {
        store_at = first_mem;
    }
    if op == "xchg"
        && let Some(Opnd::Mem(m)) = ops.get(1)
    {
        store_at = Some(*m);
    }
    if let Some(m) = store_at {
        let value = match (op, ops.get(1)) {
            ("mov", Some(Opnd::Imm(v))) => Stored::Const(*v),
            ("mov", Some(Opnd::Reg(r))) => match regs[*r] {
                V::Const(c) => Stored::Const(c),
                _ => Stored::Live,
            },
            ("mov", Some(Opnd::Sub(r, w))) => match regs[*r] {
                V::Const(c) => Stored::Const(if *w == 1 { c & 0xff } else { c & 0xffff }),
                _ => Stored::Live,
            },
            ("and", Some(Opnd::Imm(0))) => Stored::Const(0),
            ("and", Some(Opnd::Reg(r))) if regs[*r] == V::Const(0) => Stored::Const(0),
            _ => Stored::Live,
        };
        record_store(geo, regs, insn, start, &m, value, a, fields);
        if let Some(k) = frame_slot(regs, &m) {
            let v = match (op, ops.get(1)) {
                ("mov", Some(src)) => val(src, regs),
                _ => V::Other,
            };
            regs.set_slot(k, v);
        } else if points_at_stack(regs, &m)
            && let ("mov", Some(src)) = (op, ops.get(1))
            && matches!(val(src, regs), V::Leader { .. } | V::Top)
        {
            a.spills = true;
        }
        // A leader pointer written to memory leaves the function.
        if op == "mov"
            && let Some(Opnd::Reg(r)) = ops.get(1)
            && let V::Leader { off, exact, .. } = regs[*r]
            && !points_at_stack(regs, &m)
        {
            a.escapes.push(Escape {
                at: insn.at,
                func: start,
                kind: EscapeKind::Heap,
                off,
                exact,
                callee: None,
                via: insn.at,
            });
        }
    }
    if is_string_store(op, &insn.args)
        && let V::Leader { off, exact, .. } = regs[EDI]
    {
        a.escapes.push(Escape {
            at: insn.at,
            func: start,
            kind: EscapeKind::String,
            off,
            exact,
            callee: None,
            via: insn.at,
        });
    }

    // Register transfer.
    let dst = ops.first().copied();
    match op {
        "push" => {
            if let Some(o) = ops.first() {
                let v = val(o, regs);
                if matches!(v, V::Leader { .. }) {
                    a.spills = true;
                }
                if matches!(v, V::Leader { .. }) {
                    pushes.push((insn.at, v));
                }
            }
        }
        "call" => {
            let callee = insn.target();
            let ci = callee.and_then(|c| infos.get(&c));
            let leader_callee = ci.is_some_and(|c| c.leader_class);
            for &(at, v) in pushes.iter() {
                if let V::Leader { off, exact, .. } = v {
                    a.escapes.push(Escape {
                        at,
                        func: start,
                        kind: EscapeKind::Push,
                        off,
                        exact,
                        callee,
                        via: insn.at,
                    });
                }
            }
            pushes.clear();
            let helper = ci.is_some_and(|c| c.helper);
            if let V::Leader { off, exact, .. } = regs[ECX] {
                let this_call = ci.is_some_and(|c| c.thiscall);
                if this_call && !helper && !(leader_callee && off == 0) {
                    a.escapes.push(Escape {
                        at: insn.at,
                        func: start,
                        kind: EscapeKind::This,
                        off,
                        exact,
                        callee,
                        via: insn.at,
                    });
                }
            }
            if helper {
                regs[EAX] = V::Top;
                if ci.is_some_and(|c| c.prolog) {
                    regs[EBP] = V::Stack;
                }
            } else {
                // Under the map's own names only three functions return a
                // leader (`returns_leader`), none of them virtual: an
                // indirect call returns no leader, a decorated callee
                // returns one only when its name says so, and an
                // undecorated one (`_memset` returns its argument) is
                // unknown.
                regs[EAX] = match (callee, ci) {
                    (_, Some(c)) if c.returns_leader => leader(0, false),
                    (_, Some(c)) if c.decorated => V::Other,
                    (None, _) => V::Other,
                    _ => V::Top,
                };
                // Dead after a call, or a 64-bit return's high half.
                regs[ECX] = V::Other;
                regs[EDX] = V::Other;
            }
        }
        "mov" => match (dst, ops.get(1)) {
            (Some(Opnd::Reg(r)), Some(src)) => {
                regs[r] = val(src, regs);
                if r == ESP {
                    regs[ESP] = V::Stack;
                }
            }
            (Some(Opnd::Sub(r, _)), _) => regs[r] = V::Other,
            _ => {}
        },
        "lea" => {
            if let (Some(Opnd::Reg(r)), Some(Opnd::Mem(m))) = (dst, ops.get(1)) {
                regs[r] = address(geo, regs, m);
            }
        }
        "add" | "sub" => {
            if let (Some(Opnd::Reg(r)), Some(src)) = (dst, ops.get(1)) {
                let b = val(src, regs);
                regs[r] = arith(geo, regs[r], b, op == "sub");
            } else if let Some(Opnd::Sub(r, _)) = dst {
                regs[r] = V::Other;
            }
        }
        "xor" => match (dst, ops.get(1)) {
            (Some(Opnd::Reg(r)), Some(Opnd::Reg(s))) if r == *s => regs[r] = V::Const(0),
            (Some(Opnd::Reg(r)), _) => regs[r] = V::Other,
            (Some(Opnd::Sub(r, _)), _) => regs[r] = V::Other,
            _ => {}
        },
        "and" => match (dst, ops.get(1)) {
            (Some(Opnd::Reg(r)), Some(Opnd::Imm(0))) => regs[r] = V::Const(0),
            (Some(Opnd::Reg(r)), _) if r == ESP => regs[r] = V::Stack,
            (Some(Opnd::Reg(r)), _) => regs[r] = V::Other,
            (Some(Opnd::Sub(r, _)), _) => regs[r] = V::Other,
            _ => {}
        },
        "or" => match (dst, ops.get(1)) {
            (Some(Opnd::Reg(r)), Some(Opnd::Imm(v))) if *v == u32::MAX => {
                regs[r] = V::Const(u32::MAX)
            }
            (Some(Opnd::Reg(r)), _) => regs[r] = V::Other,
            (Some(Opnd::Sub(r, _)), _) => regs[r] = V::Other,
            _ => {}
        },
        "xchg" => match (dst, ops.get(1).copied()) {
            (Some(Opnd::Reg(r)), Some(Opnd::Reg(s))) => regs.swap(r, s),
            (Some(Opnd::Reg(r)), Some(Opnd::Mem(m))) | (Some(Opnd::Mem(m)), Some(Opnd::Reg(r))) => {
                regs[r] = load(ctx, regs, &m);
            }
            (Some(Opnd::Sub(r, _)), _) | (_, Some(Opnd::Sub(r, _))) => regs[r] = V::Other,
            _ => {}
        },
        "pop" => {
            if let Some(Opnd::Reg(r)) = dst {
                regs[r] = if r == ESP { V::Stack } else { V::Top };
            }
        }
        "leave" => {
            regs[ESP] = V::Stack;
            regs[EBP] = V::Top;
        }
        "cdq" | "cwd" => regs[EDX] = V::Other,
        "cwde" | "cbw" | "lahf" | "salc" | "xlatb" => regs[EAX] = V::Other,
        "cpuid" | "rdtsc" | "xgetbv" => {
            for r in [EAX, ECX, EDX, 3] {
                regs[r] = V::Other;
            }
        }
        "mul" | "div" | "idiv" | "imul" if ops.len() == 1 => {
            regs[EAX] = V::Other;
            regs[EDX] = V::Other;
        }
        _ if op.starts_with("cmov") => {
            if let (Some(Opnd::Reg(r)), Some(src)) = (dst, ops.get(1)) {
                regs[r] = merge(regs[r], val(src, regs));
            }
        }
        _ if string_op(op, &insn.args) => {
            // `stos` and `scas` move `edi`, `lods` moves `esi`, `movs` and
            // `cmps` move both.
            if !op.contains("lods") {
                regs[EDI] = V::Top;
            }
            if !op.contains("stos") && !op.contains("scas") {
                regs[ESI] = V::Top;
            }
            if op.starts_with("rep") {
                regs[ECX] = V::Top;
            }
            if op.contains("lods") {
                regs[EAX] = V::Other;
            }
        }
        _ => {
            if !reads_first(op) {
                match dst {
                    Some(Opnd::Reg(r)) => {
                        regs[r] = if r == ESP { V::Stack } else { V::Other };
                    }
                    Some(Opnd::Sub(r, _)) => regs[r] = V::Other,
                    _ => {}
                }
            }
        }
    }
}

/// `[ebp + k]` while `ebp` is the frame, with no index: the slot `k`.
fn frame_slot(regs: &Regs, m: &Mem) -> Option<u32> {
    (m.base == Some(EBP) && m.index.is_none() && regs[EBP] == V::Stack && !m.seg).then_some(m.disp)
}

fn points_at_stack(regs: &Regs, m: &Mem) -> bool {
    m.base.is_some_and(|b| b == ESP || regs[b] == V::Stack)
}

/// The value a load from `m` gives.
fn load(ctx: &Context, regs: &Regs, m: &Mem) -> V {
    if m.seg {
        return V::Other;
    }
    if m.base.is_none() && m.index.is_none() {
        return if ctx.refs.contains(&m.disp) {
            leader(0, true)
        } else {
            V::Other
        };
    }
    if let Some(k) = frame_slot(regs, m) {
        // A spill holds what was stored; a local nothing stored a leader or
        // an unknown into holds neither; an argument (`[ebp + 8]` up) is
        // whatever the caller pushed.
        return match regs.slot(k) {
            Some(v) => v,
            None if (k as i32) < 0 => V::Other,
            None => V::Arg,
        };
    }
    if points_at_stack(regs, m) {
        // An argument or a spill off `esp`: whatever was stored there.
        return V::Arg;
    }
    V::Other
}

/// The address `lea` computes.
fn address(geo: &Geometry, regs: &Regs, m: &Mem) -> V {
    if let V::Leader { off, .. } = geo.norm(m.disp) {
        // `leaders` plus a slot multiple, or plus an index.
        return leader(off, m.base.is_none() && m.index.is_none());
    }
    let base = m.base.map_or(V::Const(0), |b| regs[b]);
    let index = m.index.map(|(r, s)| (regs[r], s));
    match (base, index) {
        (V::Leader { off, span, exact }, idx) => V::Leader {
            off: off.wrapping_add(m.disp) % LEADER_SIZE,
            span,
            exact: exact && idx.is_none(),
        },
        (V::Const(c), None) => geo.norm(c.wrapping_add(m.disp)),
        (V::Const(c), Some((V::Const(i), s))) => geo.norm(
            c.wrapping_add(m.disp)
                .wrapping_add(i.wrapping_mul(s as u32)),
        ),
        (_, Some((V::Leader { off, span, .. }, 1))) => V::Leader {
            off: off.wrapping_add(m.disp) % LEADER_SIZE,
            span,
            exact: false,
        },
        (V::Stack, _) => V::Stack,
        (V::Top, _) | (_, Some((V::Top, _))) => V::Top,
        (V::Arg, _) => V::Arg,
        _ => V::Other,
    }
}

fn arith(geo: &Geometry, a: V, b: V, sub: bool) -> V {
    use V::*;
    match (a, b, sub) {
        (Stack, _, _) => Stack,
        (Leader { off, span, exact }, Const(c), false) => Leader {
            off: off.wrapping_add(c) % LEADER_SIZE,
            span,
            exact,
        },
        (Leader { off, span, exact }, Const(c), true) => Leader {
            off: off.wrapping_add(LEADER_SIZE - c % LEADER_SIZE) % LEADER_SIZE,
            span,
            exact,
        },
        (Const(x), Const(y), false) => geo.norm(x.wrapping_add(y)),
        (Const(x), Const(y), true) => geo.norm(x.wrapping_sub(y)),
        (Const(_) | Other | Top, Leader { off, span, .. }, false)
        | (Leader { off, span, .. }, Other | Top, _) => Leader {
            off,
            span,
            exact: false,
        },
        (Leader { .. }, Leader { .. }, true) => Other,
        (Top, _, _) => Top,
        (Arg, _, _) => Arg,
        _ => Other,
    }
}

#[allow(clippy::too_many_arguments)]
fn record_store(
    geo: &Geometry,
    regs: &Regs,
    insn: &Insn,
    start: u32,
    m: &Mem,
    value: Stored,
    a: &mut Analysis,
    fields: &[Field],
) {
    if m.seg {
        return;
    }
    let reached = !a.unreached;
    let site = |off: u32, span: u32, exact: bool| Site {
        at: insn.at,
        func: start,
        op: insn.op.clone(),
        args: insn.args.clone(),
        off,
        width: m.width + span,
        exact,
        value,
        reached,
        arg: false,
    };
    // An absolute address in `leaders.list`, with or without a slot index.
    if let V::Leader { off, .. } = geo.norm(m.disp) {
        let exact = m.base.is_none() && m.index.is_none();
        for f in fields {
            if overlaps(off, m.width, f) {
                a.direct.push((f.name, site(off, 0, exact)));
            }
        }
        return;
    }
    let Some(b) = m.base else {
        return;
    };
    let based = match (regs[b], m.index) {
        (V::Leader { off, span, exact }, idx) => Some((
            off.wrapping_add(m.disp) % LEADER_SIZE,
            span,
            exact && idx.is_none(),
        )),
        (_, Some((r, 1))) => match regs[r] {
            V::Leader { off, span, .. } => {
                Some((off.wrapping_add(m.disp) % LEADER_SIZE, span, false))
            }
            _ => None,
        },
        _ => None,
    };
    if let Some((off, span, exact)) = based {
        // A pointer a loop walks is charged to the element it starts at:
        // the walk stays in the array it starts in (`int[8]` by the slot,
        // in every loop the targets' neighbourhoods hold).
        for f in fields {
            if overlaps(off, m.width, f) {
                a.direct.push((f.name, site(off, 0, exact && span == 0)));
            }
        }
        return;
    }
    if matches!(regs[b], V::Top | V::Arg)
        && b != ESP
        && fields.iter().any(|f| overlaps(m.disp, m.width, f))
    {
        let mut s = site(m.disp, 0, m.index.is_none());
        s.arg = regs[b] == V::Arg;
        a.raw.push((m.disp, s));
    }
}

// ---------------------------------------------------------------------------
// Running it on an install.

/// The `llvm-objdump` to run: `$LLVM_OBJDUMP`, else the first on `PATH`,
/// else Homebrew's keg.
pub fn objdump() -> Option<String> {
    if let Ok(p) = std::env::var("LLVM_OBJDUMP") {
        return Some(p);
    }
    let path = std::env::var("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|d| d.join("llvm-objdump"))
        .chain(std::iter::once(std::path::PathBuf::from(
            "/opt/homebrew/opt/llvm/bin/llvm-objdump",
        )))
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

/// What [`scan_install`] read besides the scan.
#[derive(Debug)]
pub struct Image {
    pub leaders: u32,
    /// The two references' addresses, each checked to hold `leaders`.
    pub refs: BTreeSet<u32>,
    /// Every 4-aligned word outside `.text` that points into
    /// `leaders.list`, by address.
    pub static_pointers: Vec<(u32, u32)>,
    /// Every name of the map whose decoration returns a pointer or a
    /// reference to a leader: the calls whose `eax` the scan holds as one.
    pub leader_returning: Vec<String>,
}

/// Scans `riseofnations.exe` under `root` for `fields`, reading its listing
/// from `objdump`. `seed_this: false` is the perturbation: the leader
/// methods' `this` goes unknown.
pub fn scan_install(
    root: &str,
    objdump: &str,
    fields: &[Field],
    seed_this: bool,
) -> Result<(Scan, Image), String> {
    use std::io::BufRead;
    let exe = format!("{root}/riseofnations.exe");
    let map_text = std::fs::read(format!("{root}/sbl/rise_z.map"))
        .map_err(|e| format!("sbl/rise_z.map: {e}"))?;
    let map = Map::parse(&String::from_utf8_lossy(&map_text));
    let leaders = *map
        .data
        .get("?leaders@@3VLeaders@@A")
        .ok_or("the map names no `leaders`")?;
    let pe = crate::pe::Pe::open(&exe).ok_or("riseofnations.exe is not a PE file")?;
    let mut refs = BTreeSet::new();
    for name in [
        "?leaders@GameAccess@@2AAVLeaders@@A",
        "?leadersc@GameAccessConst@@2ABVLeaders@@B",
    ] {
        let at = *map
            .data
            .get(name)
            .ok_or(format!("the map names no `{name}`"))?;
        let held = pe
            .i32s(at, 1)
            .ok_or(format!("{name} is outside the image"))?[0] as u32;
        if held != leaders {
            return Err(format!("{name} holds {held:#x}, not leaders {leaders:#x}"));
        }
        refs.insert(at);
    }
    let text_start = map.funcs.first().map_or(0, |f| f.start);
    let static_pointers = pe.words_in(
        leaders,
        leaders + LEADERS_LIST_LEN * LEADER_SIZE,
        text_start,
    );
    let words = |va: u32, n: usize| -> Option<Vec<u32>> {
        let mut out = Vec::new();
        for i in 0..n {
            match pe.i32s(va + 4 * i as u32, 1) {
                Some(w) => out.push(w[0] as u32),
                None => break,
            }
        }
        Some(out)
    };
    let ctx = Context {
        map: &map,
        leaders,
        refs: refs.clone(),
        words: &words,
        seed_this,
    };
    let mut child = std::process::Command::new(objdump)
        .args([
            "-d",
            "--no-show-raw-insn",
            "-M",
            "intel",
            "--section=.text",
            &exe,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("{objdump}: {e}"))?;
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let lines = std::io::BufReader::new(stdout)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| Insn::parse(&l));
    let scan = scan(&ctx, lines, fields);
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("{objdump} exited {status}"));
    }
    Ok((
        scan,
        Image {
            leaders,
            refs,
            static_pointers,
            leader_returning: map
                .funcs
                .iter()
                .flat_map(|f| f.names.iter())
                .filter(|n| returns_leader(n))
                .cloned()
                .collect(),
        },
    ))
}

#[cfg(test)]
mod tests;
