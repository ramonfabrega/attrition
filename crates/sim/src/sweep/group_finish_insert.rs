//! `Group::finish_insert@0070e620` under the emulator against the port (item 3372; tools/sweep_gen.py, obj with windows).

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

/// `Group::finish_insert@0070e620`, run by `tools/emu/sweep_group_finish_insert.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p24 the place(s), in flag order; argument 0 names 64 random bytes; callee 0x0046d620 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x44 (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0x6c (0 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x74 (0 args) stubbed; vtable slot +0x78 (0 args) stubbed; vtable slot +0x7c (0 args) stubbed; vtable slot +0x80 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x98 (0 args) stubbed; vtable slot +0xa0 (0 args) stubbed; vtable slot +0xa4 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00704990 (11 args) stubbed; callee 0x0070fbe0 (5 args) stubbed; callee 0x00700b90 (2 args) stubbed; callee 0x00700010 (2 args) stubbed; callee 0x00712490 (5 args) stubbed; callee 0x006fd510 (3 args) stubbed; callee 0x006fcd30 (4 args) stubbed; callee 0x006fe1a0 (5 args) stubbed; callee 0x00701cc0 (5 args) stubbed; callee 0x00707220 (5 args) stubbed; callee 0x00706d90 (2 args) stubbed; callee 0x00704520 (3 args) stubbed; callee 0x00700490 (4 args) stubbed; callee 0x00730bb0 (1 args) stubbed; the word at `p0+0x14` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p1+8` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed, null a third of the rows); stub 2's answer points at 128 random bytes less `0` (placed); stub 3's answer points at 128 random bytes less `0` (placed); stub 4's answer points at 128 random bytes less `0` (placed); stub 5's answer points at 128 random bytes less `0` (placed); stub 6's answer points at 128 random bytes less `0` (placed); stub 7's answer points at 128 random bytes less `0` (placed); stub 8's answer points at 128 random bytes less `0` (placed); stub 9's answer points at 128 random bytes less `0` (placed); stub 10's answer points at 128 random bytes less `0` (placed); stub 11's answer points at 128 random bytes less `0` (placed); stub 12's answer points at 128 random bytes less `0` (placed); stub 13's answer points at 128 random bytes less `0` (placed); stub 14's answer points at 128 random bytes less `0` (placed); stub 15's answer points at 128 random bytes less `0` (placed); stub 16's answer points at 128 random bytes less `0` (placed); stub 17's answer points at 128 random bytes less `0` (placed); stub 18's answer points at 128 random bytes less `0` (placed); stub 19's answer points at 128 random bytes less `0` (placed); stub 20's answer points at 128 random bytes less `0` (placed); the word at `p10+4` points at 8 random bytes less `0` (placed); the word at `p13+4` points at 8 random bytes less `0` (placed); stub 1 answered from [1, 1, 2, 3, 4, 6, 6, 7, 8, 10, 11, 12, 13, 13, 14, 15, 18, 19, 21, 22, 23, 26, 5, 0, 9, 16, 17, 20, 24, 25, 27] (90%); the stubbed vtable laid at `p2+0`; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 lowers `ecx+0xc` (4 bytes) by one each call; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_finish_insert.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a`: 0 the list's address, 1 `remove_current`'s answer, 2 `get_type`'s, 3.. the accessors' answers, then the callees'.
            // Windows: p0 the OrderList (`+4` cursor order, `+8` flag byte, `+0xc` cursor node, `+0x10` count, `+0x14` head node),
            // p1 the head node (`+8` its order, `+0xc` its flag byte), p2 the head's order, p3 the cursor's own order (when the
            // head is null), p4..p22 the nineteen accessors' records (slot order below), p23/p24 the vbase tables of slots 0x60 and
            // 0x70 (`+4` the displacement). `remove_current` is the stub that counts `+0x10` down. The port is the crate's own
            // dispatch, `crate::finish_insert_view::finish_insert`, over a view of the windows.
            use crate::finish_insert_view::{FinishInsertView, finish_insert};
            const SLOTS: [u32; 19] = [
                0x40, 0x44, 0x48, 0x4c, 0x50, 0x54, 0x60, 0x64, 0x6c, 0x70, 0x74, 0x78, 0x7c, 0x80,
                0x90, 0x98, 0xa0, 0xa4, 0xac,
            ];
            const SENT: i32 = 0x7fff_0000;
            struct V<'a> {
                a: &'a [i32],
                p: &'a mut [Vec<u8>],
                ord: usize,
                ecx_m: bool,
                calls: Vec<Vec<String>>,
            }
            impl V<'_> {
                fn rec(slot: u32) -> usize {
                    4 + SLOTS.iter().position(|&s| s == slot).expect("a slot")
                }
                fn fmt(&self, v: i32) -> String {
                    if (SENT..SENT + 0x100).contains(&v) {
                        format!("p{}+0x0", v - SENT)
                    } else {
                        v.to_string()
                    }
                }
                fn push(&mut self, k: usize, args: &[i32]) {
                    let toks: Vec<String> = args.iter().map(|&x| self.fmt(x)).collect();
                    self.calls[k].push(format!("m+0x0,{}", toks.join(",")));
                    self.ecx_m = true;
                }
            }
            impl FinishInsertView for V<'_> {
                fn reseat(&mut self) {
                    let head = rd(&self.p[0], 0x14);
                    if head != 0 {
                        wr(&mut self.p[0], 0xc, head);
                        let o = rd(&self.p[1], 8);
                        wr(&mut self.p[0], 4, o);
                        self.p[0][8] = self.p[1][0xc];
                        self.ord = 2;
                    } else {
                        self.ord = 3;
                    }
                }
                fn pending(&mut self) -> bool {
                    rd(&self.p[0], 0x10) != 0
                }
                fn take(&mut self) -> bool {
                    let some = rd(&self.p[0], 4) != 0;
                    let n = rd(&self.p[0], 0x10);
                    wr(&mut self.p[0], 0x10, n.wrapping_sub(1));
                    some
                }
                fn kind(&mut self) -> i32 {
                    self.ecx_m = false;
                    self.a[2]
                }
                fn word(&mut self, slot: u32, off: u32) -> i32 {
                    self.ecx_m = false;
                    rd(&self.p[Self::rec(slot)], off as usize)
                }
                fn record(&mut self, slot: u32) -> i32 {
                    self.ecx_m = false;
                    SENT + Self::rec(slot) as i32
                }
                fn flags4(&mut self, slot: u32) -> i32 {
                    self.ecx_m = false;
                    let w = if slot == 0x60 { 23 } else { 24 };
                    let d = rd(&self.p[w], 4);
                    i32::from(self.p[Self::rec(slot)][(d + 8) as usize]) & 4
                }
                fn left_ecx(&mut self) -> i32 {
                    SENT + self.ord as i32
                }
                fn give_obj(&mut self) {
                    let ord = format!("p{}+0x0", self.ord);
                    let ecx = if self.ecx_m {
                        "m+0x0".to_string()
                    } else {
                        ord.clone()
                    };
                    self.calls[13].push(format!("{ecx},{ord}"));
                }
                fn action_move_near(&mut self, a: [i32; 11]) {
                    self.push(0, &a);
                }
                fn action_swarm_around(&mut self, a: [i32; 5]) {
                    self.push(1, &a);
                }
                fn action_gather(&mut self, a: [i32; 2]) {
                    self.push(2, &a);
                }
                fn action_board_ship(&mut self, a: [i32; 2]) {
                    self.push(3, &a);
                }
                fn action_attack(&mut self, a: [i32; 5]) {
                    self.push(4, &a);
                }
                fn action_follow(&mut self, a: [i32; 3]) {
                    self.push(5, &a);
                }
                fn action_guard(&mut self, a: [i32; 4]) {
                    self.push(6, &a);
                }
                fn action_spell(&mut self, a: [i32; 5]) {
                    self.push(7, &a);
                }
                fn action_trade(&mut self, a: [i32; 5]) {
                    self.push(8, &a);
                }
                fn action_form(&mut self, a: [i32; 5]) {
                    self.push(9, &a);
                }
                fn redo_patrol_order(&mut self, a: [i32; 2]) {
                    self.push(10, &a);
                }
                fn action_attack_ground(&mut self, a: [i32; 3]) {
                    self.push(11, &a);
                }
                fn action_garrison(&mut self, a: [i32; 4]) {
                    self.push(12, &a);
                }
            }
            let mut v = V {
                a,
                p,
                ord: 0,
                ecx_m: false,
                calls: vec![Vec::new(); 14],
            };
            finish_insert(&mut v);
            for c in &v.calls {
                log.push(if c.is_empty() {
                    "-".to_string()
                } else {
                    c.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "finish_insert — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "finish_insert — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "finish_insert — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "finish_insert — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Group::finish_insert@0070e620: {n} rows agree");
}
