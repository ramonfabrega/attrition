//! `Unit::go_around_building@005fc350` under the emulator against the port (item 3330; tools/sweep_gen.py, obj with windows).

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

/// `Unit::go_around_building@005fc350`, run by `tools/emu/sweep_unit_go_around_building.py` under unicorn
/// (tools/emu): 344 random bytes of object, 4 argument(s), `p` windows in order: p0..p2 the place(s), in flag order, p3 stub 2's answer window; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; callee 0x00607c30 (8 args) stubbed; callee 0x0046d820 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 0, 1, 1, 1] (90%); stub 2 answers the address of 92 random bytes, a `p` window after the places; stub 0 answers 40 values in call order, the last from [0], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_go_around_building.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // The recorded `invalid_loc` calls end in the caller's stale `ecx` (a register, not an argument the
        // callee reads): blanked. The push's argument is a stack local whose address moves 0x58 a recursion
        // level (a frame, not a state): blanked too.
        let want_log: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s0:") {
                Some(r) => format!("s0:{}*", &r[..r.rfind(',').expect("a comma") + 1]),
                None if t.starts_with("s1:") => "s1:m+0xb8,sp-*".to_string(),
                None => t.to_string(),
            })
            .collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // **The port is the crate's own steps**,
            // `crate::detour::go_around_building_steps`, which
            // `Sim::go_around_building` runs over its `SimDetour`; here the
            // view is the emulated original's windows.
            //
            // `a`: the four stack arguments (`last.x`, `last.y`, the angle, the
            // speed), the three stubs' single answers (`a[4..7]`, unused: the
            // slot-0x40 stub's answer is `p3`'s address and the push has none),
            // then stub 0's forty call-order answers (`a[7..47]`, the last
            // repeated). Stubs: 0 `UnitData::invalid_loc` (a callee, eight
            // stack arguments: the tile pair, five zeros and the caller's stale
            // `ecx`, which the recorded call carries and the port cannot: it
            // is blanked on both sides), 1 `Stack<PathData>::push` (`ecx` =
            // `this+0xb8`, its argument a stack local whose 16 bytes the row
            // prints after the calls), 2 the current order's slot `+0x40` (the
            // `MoveOrder`, `p3`). Windows: `m` the unit (`+0x10`/`+0x14` the
            // scrambled position, `+0x68` the masks, `+0xaf` the recursion
            // depth, `+0xdc` the order list's head), `c0` `div_3_table`, `c1`
            // the world (`+0x18`/`+0x1c` its extent in tiles), `p0` the head
            // node (`+4` its prev), `p1` that node (`+8` its order, `+0xc` its
            // metric), `p2` the order, `p3` the `MoveOrder`.
            use crate::Pos;
            use crate::detour::{DetourView, go_around_building_steps};
            use crate::movement::Angle;
            use crate::orders::PathData;
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            fn rd16(w: &[u8], o: usize) -> i32 {
                i32::from(i16::from_le_bytes(
                    w[o..o + 2].try_into().expect("two bytes"),
                ))
            }
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                pushed: Vec<String>,
                turn: usize,
            }
            impl DetourView for V<'_> {
                fn refuses(&mut self, t: Pos) -> bool {
                    self.log
                        .push(format!("s0:m+0x0,{},{},0,0,0,0,0,*", t.x, t.y));
                    let i = self.turn.min(39);
                    self.turn += 1;
                    self.a[7 + i] != 0
                }
                fn here(&self) -> Pos {
                    Pos::new(rd32(self.m, 0x10) ^ 0x63637, rd32(self.m, 0x14) ^ 0x63637)
                }
                fn world_tiles(&self) -> (i32, i32) {
                    (rd32(&self.c[1], 0x18), rd32(&self.c[1], 0x1c))
                }
                fn waypoint(&self) -> Pos {
                    Pos::new(rd32(&self.p[3], 0x2c), rd32(&self.p[3], 0x30))
                }
                fn off(&self) -> Pos {
                    Pos::new(rd16(&self.p[3], 0x4c), rd16(&self.p[3], 0x4e))
                }
                fn push(&mut self, e: PathData) {
                    self.log.push("s1:m+0xb8,sp-*".to_string());
                    let mut hex = String::new();
                    for w in [e.to.x, e.to.y, e.tolerance, i32::from(e.flags)] {
                        for b in w.to_le_bytes() {
                            hex.push_str(&format!("{b:02x}"));
                        }
                    }
                    self.pushed.push(hex);
                }
                fn gave_up(&mut self) {
                    self.p[3][0x10..0x14].copy_from_slice(&0i32.to_le_bytes());
                    let masks = rd32(self.m, 0x68) & !8;
                    self.m[0x68..0x6c].copy_from_slice(&masks.to_le_bytes());
                    self.m[0xaf] = 10;
                }
            }
            // The head's cursor caching, before anything else: `+0xd4` the
            // head's prev node, `+0xcc` that node's order, `+0xd0` its metric.
            let prev: [u8; 4] = p[0][4..8].try_into().expect("four bytes");
            let data: [u8; 4] = p[1][8..12].try_into().expect("four bytes");
            m[0xd4..0xd8].copy_from_slice(&prev);
            m[0xcc..0xd0].copy_from_slice(&data);
            m[0xd0] = p[1][0xc];
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                pushed: Vec::new(),
                turn: 0,
            };
            go_around_building_steps(&mut v, Pos::new(a[0], a[1]), Angle(a[2]), a[3]);
            let pushed = if v.pushed.is_empty() {
                "-".to_string()
            } else {
                v.pushed.join("/")
            };
            // No call recorded at all is a `-` column of its own.
            if v.log.is_empty() {
                v.log.push("-".to_string());
            }
            v.log.push(pushed);
            want_eax
        };
        let _ = (got, want_eax);
        if let Some(i) = (0..log.len().max(want_log.len())).find(|&i| log.get(i) != want_log.get(i))
        {
            let args = line
                .split(" | ")
                .next()
                .unwrap_or("")
                .split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" ");
            panic!(
                "first parts at call {i}: port {:?} original {:?}; args {args}; counts {} vs {}",
                log.get(i),
                want_log.get(i),
                log.len(),
                want_log.len()
            );
        }
        assert_eq!(
            log, want_log,
            "go_around_building — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "go_around_building — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "go_around_building — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "go_around_building — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::go_around_building@005fc350: {n} rows agree");
}

fn a_dbg(line: &str) -> String {
    line.split(" | ")
        .next()
        .unwrap_or("")
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}
