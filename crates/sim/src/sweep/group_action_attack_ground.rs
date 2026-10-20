//! `Group::action_attack_ground@00704520` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_attack_ground@00704520`, run by `tools/emu/sweep_group_action_attack_ground.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 3 argument(s), `p` windows in order: p0..p28 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 64 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; vtable slot +0xc (4 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; callee 0x006e1200 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x0070c450 (0 args) stubbed; callee 0x0061db20 (0 args) stubbed; callee 0x005e4290 (6 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x00703d80 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `p5+12` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 256 random bytes less `0` (placed); the word at `p10+4` points at 256 random bytes less `0` (placed); the word at `p10+8` points at 256 random bytes less `0` (placed); the word at `p10+12` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 256 random bytes less `0` (placed); the word at `p15+4` points at 256 random bytes less `0` (placed); the word at `p15+8` points at 256 random bytes less `0` (placed); the word at `p15+12` points at 256 random bytes less `0` (placed); the word at `c2+0x134` points at 1024 random bytes less `0` (placed); the word at `p11+0x18` points at 64 random bytes less `0` (placed); the word at `p12+0x18` points at 64 random bytes less `0` (placed); the word at `p13+0x18` points at 64 random bytes less `0` (placed); the word at `p14+0x18` points at 64 random bytes less `0` (placed); the word at `p16+0x18` points at 64 random bytes less `0` (placed); the word at `p17+0x18` points at 64 random bytes less `0` (placed); the word at `p18+0x18` points at 64 random bytes less `0` (placed); the word at `p19+0x18` points at 64 random bytes less `0` (placed); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [1, 1, 0] (90%); stub 5 answered from [0, 0, 0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 11 answered from [0, 0, 1] (90%); stub 12 answered from [0, 1, 1] (90%); stub 13 answered from [1, 1, 1, 0] (90%); stub 3 answers 16 values in call order, printed after the answers; stub 4 answers 16 values in call order, printed after the answers; stub 5 answers 16 values in call order, printed after the answers; stub 13 answers 16 values in call order, printed after the answers; stub 10 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; stub 16 writes inc (4 bytes) at `ecx+0xc` each call; figures p1..p4 link at +0x28 (2 bytes) below their own index or to -1; figures p6..p9 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_attack_ground.py") else {
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
            {
                // Window layout. `m` the group (`+0xc` count, `+0x49` the flag, `+0x4a` player, the shorts at
                // `+0x8cc`); `c[0]` the objects cell, `c[1]` the console (`+0x298` who), `c[2]` the world
                // (`+0`, `+4` its cells), `c[3]` `div_3_table`, `c[4]` `ignore_orders` (held at 0: the head's
                // list is `group_action_attack_ground_head`'s). `p[1 + o]` and `p[6 + o]` the objects of
                // players 0 and 1, `p[11 + o]` and `p[16 + o]` their units, `p[21 + o]` and `p[25 + o]` the
                // units' types, `p[20]` the tile table. `a`: the three arguments, then every stub's answer
                // (`a[3 + k]`), then the four `--seq` runs (`SEQ_AT`).
                //
                // **The port is the crate's own steps**, `crate::attack_ground_view::action_attack_ground_steps`.
                // The scratch group's list is a stack local, not a window: the stubbed `Group::add` counts
                // its calls (the generator's `--stub-write ecx+0xc:inc`) and writes no member, so the
                // members the port re-adds read `(0, 0)` here.
                use crate::attack_ground_view::{AttackGroundView, action_attack_ground_steps};
                const SEQ_AT: [(usize, usize); 4] = [(3, 21), (4, 37), (5, 53), (13, 69)];
                const MESSAGE_WIN: i32 = 15_267_872;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 14],
                    scratch: i32,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                impl V<'_> {
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(15);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                    fn ow(who: i32, o: i32) -> i32 {
                        1 + 5 * who + o
                    }
                    fn obj(&self, who: i32, o: i32) -> &[u8] {
                        &self.p[Self::ow(who, o) as usize]
                    }
                    fn obj_n(who: i32, o: i32) -> String {
                        format!("p{}+0x0", Self::ow(who, o))
                    }
                    fn unit_n(who: i32, o: i32) -> String {
                        format!("p{}+0x0", Self::ow(who, o) + 10)
                    }
                    fn type_n(who: i32, o: i32) -> String {
                        format!("p{}+0x0", 21 + 4 * who + o)
                    }
                }
                impl AttackGroundView for V<'_> {
                    fn refresh(&mut self) {
                        self.rec(0, "m+0x0", &[]);
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[4], 0) != 0
                    }
                    fn ignoring_count(&self, _who: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn drop_ignored(&mut self, _o: i32, _who: i32) {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn world_dims(&self) -> (i32, i32) {
                        (rd32(&self.c[2], 0), rd32(&self.c[2], 4))
                    }
                    fn tile_owner(&self, x: i32, y: i32) -> i32 {
                        let (w, _) = self.world_dims();
                        let d3 = |v: i32| rd32(&self.c[3], 4 * (v >> 8) as usize);
                        let row = w.wrapping_mul(d3(y)).wrapping_add(d3(x));
                        i32::from(self.p[20][(row as usize) * 0x1c + 0xf] as i8)
                    }
                    fn is_peace(&mut self, who: i32, owner: i32) -> bool {
                        self.rec(6, &format!("{}", 0x00e3_a390 + 0x6eec * who), &[owner]);
                        self.a[3 + 6] != 0
                    }
                    fn console_who(&self) -> i32 {
                        rd32(&self.c[1], 0x298)
                    }
                    fn refuse(&mut self) {
                        self.log.push("s7:sp-0xa2c,33700".into());
                        self.log
                            .push(format!("s8:{MESSAGE_WIN},0,0,0,0,0,-1,13161032"));
                        self.log.push(format!("s9:{MESSAGE_WIN},64"));
                    }
                    fn flagged(&self) -> bool {
                        self.m[0x49] != 0
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn find_leader(&mut self) -> i32 {
                        self.rec(10, "m+0x0", &[0]);
                        self.a[3 + 10]
                    }
                    fn can_carry_air(&mut self, who: i32, o: i32) -> bool {
                        self.rec(11, &Self::obj_n(who, o), &[2]);
                        self.a[3 + 11] != 0
                    }
                    fn obj_c0(&mut self, who: i32, o: i32) -> bool {
                        self.rec(5, &Self::obj_n(who, o), &[]);
                        self.seq(5) != 0
                    }
                    fn is_on_map(&mut self) -> bool {
                        self.rec(12, "m+0x0", &[]);
                        self.a[3 + 12] != 0
                    }
                    fn obj_slot8(&mut self, who: i32, o: i32) -> bool {
                        self.rec(3, &Self::obj_n(who, o), &[]);
                        self.seq(3) != 0
                    }
                    fn obj_slot_bc(&mut self, who: i32, o: i32) -> bool {
                        self.rec(4, &Self::obj_n(who, o), &[]);
                        self.seq(4) != 0
                    }
                    fn unit_c0(&mut self, who: i32, o: i32) -> bool {
                        self.rec(5, &Self::unit_n(who, o), &[]);
                        self.seq(5) != 0
                    }
                    fn unit_can_attack_ground(&mut self, who: i32, o: i32) -> bool {
                        self.rec(13, &Self::type_n(who, o), &[]);
                        self.seq(13) != 0
                    }
                    fn add_attack_ground_order(
                        &mut self,
                        who: i32,
                        o: i32,
                        x: i32,
                        y: i32,
                        queue: i32,
                    ) {
                        self.rec(14, &Self::unit_n(who, o), &[x, y, queue, 1, 0, 0]);
                    }
                    fn obj_flag1(&self, who: i32, o: i32) -> bool {
                        self.obj(who, o)[8] & 1 != 0
                    }
                    fn obj_link(&self, who: i32, o: i32) -> i32 {
                        rd16(self.obj(who, o), 0x28)
                    }
                    fn obj_link_who(&self, who: i32, o: i32) -> i32 {
                        i32::from(self.obj(who, o)[0x3e] as i8)
                    }
                    fn scratch_clear(&mut self) {
                        self.rec(15, "sp-0xa04", &[-1]);
                    }
                    fn scratch_add(&mut self, id: i32, who: i32) {
                        self.rec(16, "sp-0xa04", &[id, who, 0, 0]);
                        self.scratch += 1;
                    }
                    fn scratch_count(&self) -> i32 {
                        self.scratch
                    }
                    fn scratch_member(&self, _k: i32) -> (i32, i32) {
                        (0, 0)
                    }
                    fn clear(&mut self) {
                        self.rec(15, "m+0x0", &[-1]);
                    }
                    fn add_member(&mut self, id: i32, who: i32) {
                        self.rec(2, "m+0x0", &[id, who, 1, 1]);
                    }
                    fn air_attack_ground(&mut self, x: i32, y: i32, queue: i32) {
                        self.rec(17, "m+0x0", &[x, y, queue]);
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 14],
                    scratch: 0,
                };
                action_attack_ground_steps(&mut v, [a[0], a[1], a[2]]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_attack_ground — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "action_attack_ground — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_attack_ground — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "action_attack_ground — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Group::action_attack_ground@00704520: {n} rows agree");
}
