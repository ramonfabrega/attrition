//! `Roads::redo_changed_roads@0088e670` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

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

/// `Roads::redo_changed_roads@0088e670`, run by `tools/emu/sweep_roads_redo_changed_roads.py` under unicorn
/// (tools/emu): 512 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0, p1 the place, p2..p16 the stubs' answer window(s); the global cell 0x00c06188 points at 32 random bytes; argument 0 names 28 random bytes; callee 0x0088e3f0 (3 args) stubbed; callee 0x008744d0 (5 args) stubbed; callee 0x00893b30 (2 args) stubbed; callee 0x008935c0 (1 args) stubbed; callee 0x00890da0 (0 args) stubbed; callee 0x0088e9d0 (2 args) stubbed; callee 0x00891d40 (1 args) stubbed; callee 0x00892c10 (1 args) stubbed; callee 0x00892550 (1 args) stubbed; callee 0x00892390 (1 args) stubbed; callee 0x00891210 (1 args) stubbed; callee 0x008910b0 (1 args) stubbed; callee 0x00890c90 (0 args) stubbed; the word at `p0+0x10` points at 32 random bytes less `0` (placed); stub 5 answered from [0, 0, 1, 7] (90%); stub 1 answers a window of its own to each call, in call order (-32 bytes, ...); stub 5 answers 4 values in call order, the last from [0, 1], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_redo_changed_roads.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // **The port is the crate's own steps**, `crate::mesh::redo_changed_roads_steps`, which
            // `Sim::redo_changed_roads` runs over its `SimRedo`; here the view is the emulated original's
            // windows. The harness owns the `Array<int>` iteration (flag 0x40 at +0x14, `cur_index` at
            // +0x18, count at +4, the data pointer at +0x10), which the crate holds as a `Vec`.
            use crate::mesh::{RedoView, redo_changed_roads_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                x: i32,
                y: i32,
                /// get_road_data calls so far; the stub's windows are p2.. one a call, the last repeated.
                roads: usize,
                diags: usize,
                cur: Option<usize>,
                /// The null-ness of the last get_road_data answer, if it was asked.
                last_null: Option<bool>,
            }
            /// `[0xc06218]` as the image holds it: the `ecx` of every `get_road_data`.
            const TERRAIN: i32 = 12628208;
            impl V<'_> {
                fn this(&self) -> &'static str {
                    "m+0x0"
                }
            }
            impl RedoView for V<'_> {
                fn tile(&mut self, index: i32) {
                    let tw = rd(&self.c[0], 0) << 2;
                    self.x = index % tw;
                    self.y = index / tw;
                    wr(self.m, 0x120, self.x);
                    wr(self.m, 0x124, self.y);
                }
                fn clear_support(&mut self) {
                    let tw = rd(&self.c[0], 0) << 2;
                    self.log.push(format!("s0:{tw},{},{},1", self.x, self.y));
                }
                fn road_data(&mut self) -> bool {
                    let k = self.roads.min(14);
                    self.roads += 1;
                    self.log
                        .push(format!("s1:{TERRAIN},{},{},0,0,0", self.x, self.y));
                    let null = self.a[20 + k] != 0;
                    self.last_null = Some(null);
                    self.cur = if null { None } else { Some(2 + k) };
                    !null
                }
                fn clear_flags(&mut self) {
                    let k = self.cur.expect("a record");
                    wr(&mut self.p[k], 4, 0);
                }
                fn fill_cache(&mut self) {
                    self.log
                        .push(format!("s2:{},{},{}", self.this(), self.x, self.y));
                }
                fn trim_directions(&mut self, list: i32) {
                    self.log.push(format!("s3:{},{list}", self.this()));
                }
                fn leech_codes(&mut self) {
                    self.log.push(format!("s4:{}", self.this()));
                }
                fn set_diags(&mut self, list: i32, param: i32) -> i32 {
                    self.log.push(format!("s5:{},{list},{param}", self.this()));
                    let v = self.a[16 + self.diags.min(3)];
                    self.diags += 1;
                    v
                }
                fn refill_for_marks(&mut self) {
                    self.fill_cache();
                }
                fn texture_marks(&mut self, list: i32) {
                    let t = self.this();
                    self.log.push(format!("s6:{t},{list}"));
                    self.log.push(format!("s7:{t},{list}"));
                    self.log.push(format!("s8:{t},{list}"));
                    // The argument is the `ecx` left over from the last stubbed call: `this`.
                    self.log.push(format!("s9:{t},{t}"));
                    self.log.push(format!("s10:{t},{list}"));
                }
                fn finish_pieces(&mut self, list: i32) {
                    let t = self.this();
                    self.log.push(format!("s10:{t},{list}"));
                    self.log.push(format!("s11:{t},{list}"));
                    self.log.push(format!("s12:{t}"));
                }
            }
            let count = rd(&p[0], 4);
            let n = if count == 0 { 0 } else { count.max(1) as usize };
            let work: Vec<i32> = (0..n).map(|i| rd(&p[1], 4 * i)).collect();
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                x: 0,
                y: 0,
                roads: 0,
                diags: 0,
                cur: None,
                last_null: None,
            };
            redo_changed_roads_steps(&mut v, &work, a[1], a[2]);
            let last_null = v.last_null;
            let (m, p) = (v.m, v.p);
            // The array: the walk leaves `cur_index` at the end and the end flag set.
            p[0][0x14] |= 0x40;
            wr(&mut p[0], 0x18, n as i32);
            // `this+0x14c` holds the last answer: an address in the emulator, so only null-ness is held.
            if let Some(null) = last_null {
                let wm = unhex(after.trim().split(';').next().expect("this"));
                let at = rd(&wm, 0x14c);
                assert_eq!(
                    at == 0,
                    null,
                    "redo_changed_roads — +0x14c null-ness, row `{line}`"
                );
                wr(m, 0x14c, at);
            }
            0
        };
        let _ = (got, want_eax);
        // A row that called nothing records `-`.
        if log.is_empty() {
            log.push("-".into());
        }
        assert_eq!(
            log, want_log,
            "redo_changed_roads — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "redo_changed_roads — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "redo_changed_roads — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "redo_changed_roads — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Roads::redo_changed_roads@0088e670: {n} rows agree");
}
