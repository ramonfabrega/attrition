//! `Unit::detect_unit_collision@00617060` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Unit::detect_unit_collision@00617060`, run by `tools/emu/sweep_unit_detect_unit_collision_scan.py` under unicorn
/// (tools/emu): 256 random bytes of object, 7 argument(s), `p` windows in order: p0..p34 the place(s), in flag order, p35..p41 the stubs' answer window(s); the global cell 0x00c06188 points at 8 random bytes; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061a8 points at 32 random bytes; vtable slot +0x18 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x74 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0x4 (0 args) stubbed; vtable slot +0x0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x00682540 (8 args) stubbed; callee 0x00609fa0 (4 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x004613d0 (4 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x006488f0 (4 args) stubbed; callee 0x0060a0c0 (2 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x0060a040 (3 args) stubbed; the word at `c3+0x14` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `c3+0x30` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 272 random bytes less `0` (placed); the word at `p6+4` points at 272 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 272 random bytes less `0` (placed); the word at `p9+4` points at 272 random bytes less `0` (placed); the word at `c4+0x10` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 592 random bytes less `0` (placed); the word at `m+0xdc` points at 8 random bytes less `0` (placed); the word at `p16+4` points at 16 random bytes less `0` (placed); the word at `p17+8` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 592 random bytes less `0` (placed); the word at `p7+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p21+8` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 592 random bytes less `0` (placed); the word at `p8+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p24+4` points at 16 random bytes less `0` (placed); the word at `p25+8` points at 32 random bytes less `0` (placed); the word at `p10+0x18` points at 592 random bytes less `0` (placed); the word at `p10+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p28+4` points at 16 random bytes less `0` (placed); the word at `p29+8` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 592 random bytes less `0` (placed); the word at `p11+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p32+4` points at 16 random bytes less `0` (placed); the word at `p33+8` points at 32 random bytes less `0` (placed); slot +0xd8 laid as 0x00610af0 or its stub, a flag after the answers; slot +0x0 laid as 0x0072ff60 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1, 1] (90%); stub 1 answered from [1, 1, 1, 1, 0] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 4, 18, 19, 21, 12, 14, 10, 5, 15] (90%); stub 4 answered from [0, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [1, 1, 1, 1, 0] (90%); stub 14 answered from [0, 1, 3, 5, 7, 1, 5] (90%); stub 15 answered from [10, 10, 15, 12, 0, 1, 3] (90%); stub 16 answered from [1, 1, 0] (90%); stub 18 answered from [0, 100, 768, 960, 1000, 1728, 5000, 20000] (90%); stub 19 answered from [1, 1, 1, 0] (90%); stub 21 answered from [0, 1, 3, 5, 7] (90%); stub 5 answers the address of 80 random bytes, a `p` window after the places; stub 6 answers the address of 48 random bytes, a `p` window after the places; stub 7 answers the address of 32 random bytes, a `p` window after the places; stub 8 answers the address of 64 random bytes, a `p` window after the places; stub 20 answers the address of 64 random bytes, a `p` window after the places; stub 17 answers a window of its own to each call, in call order (64 bytes, ...); the word at `p13+0` laid as the image vtable at 0xb44478 (the stub slots copied there) half the rows; the word at `p14+0` laid as the image vtable at 0xb44478 (the stub slots copied there) half the rows; stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 16 answers 9 values in call order, printed after the answers; stub 19 answers 8 values in call order, printed after the answers; stub 21 answers 8 values in call order, printed after the answers; stub 11 answers 0..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 16 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 16 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 13 writes a value in 0..40 through its argument 5, printed after the answers (`o` in a place); stub 13 writes a value in 0..40 through its argument 6, printed after the answers (`o` in a place); stub 16 writes a value in -1..1 through its argument 2, printed after the answers (`o` in a place); stub 16 writes a value in 0..1 through its argument 3, printed after the answers (`o` in a place); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_detect_unit_collision_scan.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // **The port is `detect_collision_view::detect_unit_collision_steps`**, the walk
            // `Sim::detect_unit_collision` runs over its `SimDetect`; here the view is the emulated
            // original's windows (`Rows`, below). `a[..7]` the arguments (x, y, quick, boats, the
            // unused fifth, nocoll, top_only), `a[7..29]` the 22 stubs' answers (a stub with a
            // `--seq` answers from its sequence instead), `a[29..33]` the out-words (13: hit x,
            // y; 16: the chain head's `o`, `who`), `a[33..100]` the eight sequences, `a[100]`
            // and `a[101]` the flags that lay the image's `is_moving` (slot `0xd8`) and
            // `TargetOrder::get_target` (vtable slot 0) in the vtable, `a[102]` the flag that
            // makes `get_action`'s later calls answer null. This slice is the scan: `top_only` is
            // 1 and `quick` 0 every row (the gates are the other slice's).
            use sim::detect_collision_view::{DetectArgs, detect_unit_collision_steps};
            let args = DetectArgs {
                x: a[0],
                y: a[1],
                quick: a[2] != 0,
                boats: a[3] != 0,
                nocoll: a[5] != 0,
                top_only: a[6] != 0,
            };
            let mut view = Rows {
                a,
                m,
                c,
                p,
                log: &mut log,
                turn: [0; 22],
                ga_turn: 0,
                derefs: Default::default(),
            };
            let (eax, _) = detect_unit_collision_steps(&mut view, args);
            let derefs = view.derefs;
            if log.is_empty() {
                log.push("-".to_string());
            }
            for d in derefs {
                log.push(if d.is_empty() {
                    "-".to_string()
                } else {
                    d.iter()
                        .map(|x| {
                            x.to_le_bytes()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("/")
                });
            }
            i64::from(eax)
            // end hand
        };
        assert_eq!(got, want_eax, "detect_unit_collision — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "detect_unit_collision — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "detect_unit_collision — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "detect_unit_collision — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "detect_unit_collision — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Unit::detect_unit_collision@00617060: {n} rows agree");
}

// hand: view
use sim::detect_collision_view::{DetectCollisionView, unit_cell_of};

/// The row's windows as the view. `m` the unit; cells `c0` the world, `c1` the game (`+0x550` the
/// frame), `c2` the `div_3_table`, `c3` the objects table, `c4` the spell types; `p0..p5` the
/// objects (`p0`/`p3` a player's pointer array, `p1 p2 | p4 p5` its two objects), `p6..p11` the
/// units the same way, `p12..p14` the spell types, `p15..p18` this unit's type, list, node and
/// front order, `p19..p34` the four units' (type, list, node, order), `p35` the record slot `0x40`
/// answers, `p36` the spell record, `p37` the guard record, `p38` the target order, `p39`/`p40`
/// `get_action`'s first and later answers, `p41` `update_action`'s order.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    /// How many answers each stub has given (its sequence's cursor).
    turn: [usize; 22],
    ga_turn: usize,
    /// The bytes each `--deref` read: get_down's cx, cy; is_here's hit x, y.
    derefs: [Vec<i32>; 4],
}

/// Where each stub's sequence starts in `a`, and how long it is.
const SEQS: [(usize, usize, usize); 8] = [
    (0, 33, 8),
    (1, 41, 8),
    (2, 49, 6),
    (3, 55, 12),
    (4, 67, 8),
    (16, 75, 9),
    (19, 84, 8),
    (21, 92, 8),
];

impl Rows<'_> {
    fn seq(&mut self, stub: usize) -> i32 {
        let &(_, at, n) = SEQS.iter().find(|s| s.0 == stub).expect("a sequenced stub");
        let k = self.turn[stub].min(n - 1);
        self.turn[stub] += 1;
        self.a[at + k]
    }
    /// The window of the unit `(o, who)` (`p7 p8 | p10 p11`), its index among the four, and the
    /// windows hung off it.
    fn ui(n: (i32, i32)) -> usize {
        (2 * n.1 + n.0) as usize
    }
    fn uw(n: (i32, i32)) -> usize {
        7 + 3 * n.1 as usize + n.0 as usize
    }
    fn ow(n: (i32, i32)) -> usize {
        1 + 3 * n.1 as usize + n.0 as usize
    }
    fn utype(n: (i32, i32)) -> usize {
        19 + 4 * Self::ui(n)
    }
    fn ulist(n: (i32, i32)) -> usize {
        20 + 4 * Self::ui(n)
    }
    fn unode(n: (i32, i32)) -> usize {
        21 + 4 * Self::ui(n)
    }
    fn uorder(n: (i32, i32)) -> usize {
        22 + 4 * Self::ui(n)
    }
    fn me(&self) -> (i32, i32) {
        (
            i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]])),
            i32::from(self.m[9]),
        )
    }
    /// The cache the original refreshes before each read of an order list's front: `+0xd4` the
    /// node, `+0xcc` its order, `+0xd0` its byte. `u` is the window of the unit, `l`/`n` its list
    /// and node windows.
    fn refresh(unit: &mut [u8], list: &[u8], node: &[u8]) {
        wr(unit, 0xd4, rd(list, 4));
        wr(unit, 0xcc, rd(node, 8));
        unit[0xd0] = node[0xc];
    }
    fn refresh_me(&mut self) {
        let (l, n) = (self.p[16].clone(), self.p[17].clone());
        Self::refresh(self.m, &l, &n);
    }
}

impl DetectCollisionView for Rows<'_> {
    type Node = (i32, i32);

    fn domain(&mut self) -> i32 {
        rd(&self.p[15], 0x218)
    }
    fn type_is_siege(&mut self) -> bool {
        unreachable!("the scan slice has top_only set")
    }
    fn is_hero(&mut self) -> bool {
        unreachable!("the scan slice has top_only set")
    }
    fn is_supply(&mut self) -> bool {
        unreachable!("the scan slice has top_only set")
    }
    fn detect_boat_collision(&mut self, _x: i32, _y: i32) -> bool {
        unreachable!("the scan slice has top_only set")
    }
    fn path_detour(&mut self) -> bool {
        assert_eq!(rd(self.m, 0xc0), 0, "the scan slice has no path");
        false
    }
    fn is_safe(&mut self) -> bool {
        self.m[0xb2] != 0
    }
    fn own_cell(&mut self) -> (i32, i32) {
        (
            unit_cell_of(rd(self.m, 0x10) ^ 0x63637),
            unit_cell_of(rd(self.m, 0x14) ^ 0x63637),
        )
    }
    fn collide_here(&mut self, cell: (i32, i32), nocoll: bool) -> Option<(i32, i32)> {
        let (o, who) = self.me();
        let size = rd(&self.p[15], 0x248);
        self.log.push(format!(
            "s13:sp-0x34,{o},{who},{},{},{size},sp-0x34,sp-0x3c,{}",
            cell.0,
            cell.1,
            i32::from(nocoll)
        ));
        (self.a[7 + 13] != 0).then_some((self.a[29], self.a[30]))
    }
    fn will_be_corner(&mut self, hit: (i32, i32), cell: (i32, i32)) -> i32 {
        self.log.push(format!(
            "s14:m+0x0,{},{},{},{}",
            hit.0, hit.1, cell.0, cell.1
        ));
        self.a[7 + 14]
    }
    fn action_type(&mut self) -> i32 {
        self.log.push("s15:m+0x0".to_string());
        self.a[7 + 15]
    }
    fn attack_target_exists(&mut self) -> bool {
        // `get_action` (the first call: p39), its target order (slot `0xcc`: p38), the target's
        // `target_exists` (slot 4), and its slot 0 unless that is `TargetOrder::get_target`.
        self.log.push("s17:m+0x0".to_string());
        self.ga_turn += 1;
        self.log.push("s8:p39+0x0".to_string());
        self.log.push("s9:p38+0x0".to_string());
        let exists = self.a[7 + 9] != 0;
        if exists && self.a[101] == 0 {
            self.log.push("s10:p38+0x0".to_string());
        }
        exists
    }
    fn max_range(&mut self) -> i32 {
        self.log.push("s11:m+0x0".to_string());
        self.a[7 + 11]
    }
    fn attack_dist(&mut self, x: i32, y: i32) -> i32 {
        let (o, who) = (rd(&self.p[38], 8), rd(&self.p[38], 0xc));
        self.log.push(format!("s18:m+0x0,{o},{who},{x},{y}"));
        self.a[7 + 18]
    }
    fn world_size(&mut self) -> (i32, i32) {
        (rd(&self.c[0], 0), rd(&self.c[0], 4))
    }
    fn get_down(&mut self, cell: (i32, i32)) -> Option<Self::Node> {
        self.log
            .push("s16:c0+0x0,sp-0x4,sp-0x14,sp-0x2c,sp-0x38".to_string());
        self.derefs[0].push(cell.0);
        self.derefs[1].push(cell.1);
        (self.seq(16) != 0).then_some((self.a[31], self.a[32]))
    }
    fn node_id(&mut self, n: Self::Node) -> (i32, i32) {
        n
    }
    fn node_next(&mut self, n: Self::Node) -> Self::Node {
        let w = rd(&self.p[Self::ow(n)], 0x2c);
        (i32::from(w as i16), i32::from((w >> 16) as i16))
    }
    fn node_is_me(&mut self, n: Self::Node) -> bool {
        n == self.me()
    }
    fn node_alive(&mut self, n: Self::Node) -> bool {
        self.log.push(format!("s0:p{}+0x0", Self::ow(n)));
        self.seq(0) != 0
    }
    fn node_air(&mut self, n: Self::Node) -> bool {
        rd(&self.p[Self::utype(n)], 0x218) == 2
    }
    fn node_on_map(&mut self, n: Self::Node) -> bool {
        self.log.push(format!("s1:p{}+0x0", Self::uw(n)));
        self.seq(1) != 0
    }
    fn node_is_here(&mut self, n: Self::Node, hit: (i32, i32)) -> bool {
        self.log
            .push(format!("s19:p{}+0x0,sp-0x5c,sp-0x48", Self::uw(n)));
        self.derefs[2].push(hit.0);
        self.derefs[3].push(hit.1);
        self.seq(19) != 0
    }
    fn node_action_type(&mut self, n: Self::Node) -> i32 {
        self.log.push(format!("s17:p{}+0x0", Self::uw(n)));
        let win = if self.ga_turn == 0 { 39 } else { 40 };
        self.ga_turn += 1;
        if win == 40 && self.a[102] != 0 {
            return 0;
        }
        self.log.push(format!("s3:p{win}+0x0"));
        self.seq(3)
    }
    fn is_moving_me(&mut self) -> bool {
        if self.a[100] == 0 {
            self.log.push("s2:m+0x0".to_string());
            return self.seq(2) != 0;
        }
        if rd(self.m, 0xdc) == 0 {
            return false;
        }
        self.refresh_me();
        if rd(self.m, 0xcc) == 0 {
            return false;
        }
        self.log.push("s4:p18+0x0".to_string());
        self.seq(4) != 0
    }
    fn node_is_moving(&mut self, n: Self::Node) -> bool {
        let u = Self::uw(n);
        if self.a[100] == 0 {
            self.log.push(format!("s2:p{u}+0x0"));
            return self.seq(2) != 0;
        }
        if rd(&self.p[u], 0xdc) == 0 {
            return false;
        }
        let (l, nd) = (
            self.p[Self::ulist(n)].clone(),
            self.p[Self::unode(n)].clone(),
        );
        Self::refresh(&mut self.p[u], &l, &nd);
        if rd(&self.p[u], 0xcc) == 0 {
            return false;
        }
        self.log.push(format!("s4:p{}+0x0", Self::uorder(n)));
        self.seq(4) != 0
    }
    fn guard_targets_me(&mut self, n: Self::Node) -> bool {
        self.log.push(format!("s20:p{}+0x0", Self::uw(n)));
        self.log.push("s7:p41+0x0".to_string());
        let (o, who) = self.me();
        rd(&self.p[37], 8) == o && rd(&self.p[37], 0xc) == who
    }
    fn node_same_owner(&mut self, n: Self::Node) -> bool {
        n.1 == self.me().1
    }
    fn my_block(&mut self) -> i32 {
        rd(&self.p[15], 0x248)
    }
    fn node_block(&mut self, n: Self::Node) -> i32 {
        rd(&self.p[Self::utype(n)], 0x248)
    }
    fn groups_shared(&mut self, n: Self::Node) -> bool {
        let g = |w: &[u8]| u16::from_le_bytes([w[0x80], w[0x81]]);
        let mine = g(self.m);
        mine == g(&self.p[Self::uw(n)]) && mine != 0xffff
    }
    fn node_searching(&mut self, n: Self::Node) -> bool {
        rd(&self.p[Self::uw(n)], 0x104) != 0
    }
    fn node_front_order(&mut self, n: Self::Node) -> Option<i32> {
        let u = Self::uw(n);
        if rd(&self.p[u], 0xdc) == 0 {
            return None;
        }
        let (l, nd) = (
            self.p[Self::ulist(n)].clone(),
            self.p[Self::unode(n)].clone(),
        );
        Self::refresh(&mut self.p[u], &l, &nd);
        self.log.push(format!("s3:p{}+0x0", Self::uorder(n)));
        Some(self.seq(3))
    }
    fn node_cast_soft(&mut self, n: Self::Node) -> bool {
        let u = Self::uw(n);
        let (l, nd) = (
            self.p[Self::ulist(n)].clone(),
            self.p[Self::unode(n)].clone(),
        );
        Self::refresh(&mut self.p[u], &l, &nd);
        self.log.push(format!("s6:p{}+0x0", Self::uorder(n)));
        let idx = rd(&self.p[36], 0x20) as usize;
        let st = 13 + idx;
        if rd(&self.p[st], 0) == 0xb44478 {
            matches!(rd(&self.p[st], 4), 0x28b | 0x28d | 0x28f | 0x291)
        } else {
            self.log.push(format!("s12:p{st}+0x0"));
            self.a[7 + 12] != 0
        }
    }
    fn node_corner(&mut self, n: Self::Node, hit: (i32, i32)) -> i32 {
        self.log.push(format!(
            "s21:p{}+0x0,{},{},p{}+{:#x}",
            Self::uw(n),
            hit.0,
            hit.1,
            6 + 3 * n.1,
            4 * n.0
        ));
        self.seq(21)
    }
    fn record_hit(&mut self, n: Self::Node) {
        self.m[0x8a..0x8c].copy_from_slice(&(n.0 as i16).to_le_bytes());
        self.m[0xb3] = n.1 as u8;
        self.m[0x8c..0x8e].copy_from_slice(&[0, 0]);
    }
    fn front_is_moving(&mut self) -> bool {
        assert_ne!(rd(self.m, 0xdc), 0, "a hard hit has a front order");
        self.refresh_me();
        self.log.push("s4:p18+0x0".to_string());
        self.seq(4) != 0
    }
    fn set_move_x(&mut self, x: i32) {
        self.refresh_me();
        self.log.push("s5:p18+0x0".to_string());
        wr(&mut self.p[35], 0x3c, x);
    }
    fn set_move_y(&mut self, y: i32) {
        self.refresh_me();
        self.log.push("s5:p18+0x0".to_string());
        wr(&mut self.p[35], 0x40, y);
    }
    fn set_half_step(&mut self) {
        let w = rd(self.m, 0x68) | 0x100000;
        wr(self.m, 0x68, w);
    }
    fn clear_collider(&mut self) {
        self.m[0xb3] = 0xff;
        self.m[0x8a..0x8c].copy_from_slice(&[0xff, 0xff]);
    }
    fn collide_frame(&mut self) -> i64 {
        i64::from(rd(self.m, 0x48))
    }
    fn frame(&mut self) -> i64 {
        i64::from(rd(&self.c[1], 0x550))
    }
    fn reset_collide_count(&mut self) {
        self.m[0x88..0x8a].copy_from_slice(&[0, 0]);
    }
    fn clear_wait_flag(&mut self) {
        let w = rd(self.m, 0x68) & !0x40;
        wr(self.m, 0x68, w);
    }
}
// end hand
