//! `Roads::add_roads@0088f4b0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Roads::add_roads@0088f4b0`, run by `tools/emu/sweep_roads_add_roads.py` under unicorn
/// (tools/emu): 384 random bytes of object, 1 argument(s), `p` windows in order: p0..p3 the place(s), in flag order, p4..p27 the stubs' answer window(s); the global cell 0x00c06188 points at 1024 random bytes; the global cell 0x00c06218 points at 64 random bytes; 4 random bytes at 0x00ee19f0 (each word in 0..0); 16 random bytes at 0x00ee19f4; 8 random bytes at 0x00ee1a08; callee 0x0088fef0 (1 args) stubbed; callee 0x00893b30 (2 args) stubbed; callee 0x00866a70 (5 args) stubbed; callee 0x008744d0 (5 args) stubbed; callee 0x008935c0 (1 args) stubbed; callee 0x0088e9d0 (2 args) stubbed; callee 0x00891d40 (1 args) stubbed; callee 0x00892c10 (1 args) stubbed; callee 0x00892550 (1 args) stubbed; callee 0x00892390 (1 args) stubbed; callee 0x00891210 (1 args) stubbed; callee 0x00890c90 (0 args) stubbed; callee 0x0088e670 (3 args) stubbed; callee 0x004a3af0 (1 args) stubbed; callee 0x008e3e70 (6 args) stubbed; the word at `m+0xf0` points at 256 random bytes less `0` (placed); the word at `0xee1a04` points at 256 random bytes less `0` (placed); the word at `0x7C01002c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); stub 2 answered from [0, 1, 2, 3] (90%); stub 5 answered from [0, 1] (90%); stub 3 answers a window of its own to each call, in call order (-64 bytes, ...); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 256 bytes under the call is zeroed before each row; stub 0 writes inc (4 bytes) at `m+0xe4` each call; stub 5 writes inc (4 bytes) at `m+0xe4` each call; stub 12 writes inc (4 bytes) at `m+0x170` each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_add_roads.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout. `m` the `Roads` (`+0xe4` the queue's count, `+0xe8` its capacity, `+0xec` the step
            // `increase_size` is handed, `+0xf0` the buffer, `+0xf4` the iterator's flag byte, `+0xf8` its cursor,
            // `+0x154`/`+0x170` the two redo lists' counts); `c[0]` the world (`+0` cells wide), `c[1]` the terrain,
            // `c[2]` `_TSS0`, `c[3]` the function-static `remove_list`'s head (`+4` count, `+8` capacity, `+0xc` step),
            // `c[4]` its iterator (`+0` flag byte, `+4` cursor); `p[0]` the queue's buffer, `p[1]` the remove list's,
            // `p[2..4]` the thread-local pointer chain the `_TSS0` guard reads (never the init arm), `p[4 + k]` the
            // element the k-th `get_road_data` answers (its `+4` the flags). `a`: the argument, the stubs' answers
            // (`a[1..16]`), then one null flag a `get_road_data` call (`a[16 + k]`).
            //
            // **The port is the crate's own steps**, `crate::roads_view::add_roads_steps`, which `Sim::add_roads`
            // runs over its `SimRoads`; here the view is the emulated original's windows: the queue and the remove
            // list are loaded from them and written back, and every callee is a record.
            use crate::roads_view::{ModQueue, RoadsView, TileRec, Which, add_roads_steps};
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            fn wr32(w: &mut [u8], o: usize, v: i32) {
                w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
            fn load(buf: &[u8], count: i32, cap: i32, grow: i16, cursor: i32, flags: u8) -> ModQueue {
                ModQueue {
                    data: (0..buf.len() / 8)
                        .map(|i| (rd32(buf, 8 * i), rd32(buf, 8 * i + 4)))
                        .collect(),
                    count,
                    cap,
                    grow,
                    cursor,
                    flags,
                }
            }
            fn store(q: &ModQueue, buf: &mut [u8]) {
                for (i, &(a, b)) in q.data.iter().enumerate() {
                    if 8 * i + 8 <= buf.len() {
                        wr32(buf, 8 * i, a);
                        wr32(buf, 8 * i + 4, b);
                    }
                }
            }
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: Vec<String>,
                queue: ModQueue,
                remove: ModQueue,
                tile: (i32, i32),
                /// The window the cursor's element lies in, `p[4 + k]`.
                elem: Option<usize>,
                /// `get_road_data` calls so far.
                reads: usize,
                /// `+0x150`'s and `+0x16c`'s counts.
                redo: [i32; 2],
            }
            impl V<'_> {
                fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                    let mut t = format!("s{k}:{ecx}");
                    for v in args {
                        t.push_str(&format!(",{v}"));
                    }
                    self.log.push(t);
                }
            }
            impl RoadsView for V<'_> {
                fn queue(&mut self) -> &mut ModQueue {
                    &mut self.queue
                }
                fn remove_list(&mut self) -> &mut ModQueue {
                    &mut self.remove
                }
                fn grow(&mut self, which: Which, step: i16) {
                    let ecx = match which {
                        Which::Queue => "m+0xe0",
                        Which::Remove => "c2+0x4",
                    };
                    self.rec(13, ecx, &[i32::from(step)]);
                }
                fn world_cells(&self) -> (i32, i32) {
                    (rd32(&self.c[0], 0), rd32(&self.c[0], 4))
                }
                fn world_tiles(&self) -> (i32, i32) {
                    (rd32(&self.c[0], 0x18), rd32(&self.c[0], 0x1c))
                }
                fn set_tile(&mut self, x: i32, y: i32) {
                    self.tile = (x, y);
                    wr32(self.m, 0x120, x);
                    wr32(self.m, 0x124, y);
                }
                fn clear_roads(&mut self, _index: i32, _flags: i32) {
                    self.rec(0, "m+0x0", &[]);
                    let t = self.log.last_mut().expect("just pushed");
                    t.push_str(",sp-0x4c");
                    // The stub raises the queue's count by one: the entry is the buffer's own next slot.
                    self.queue.count += 1;
                }
                fn fill_cache(&mut self) {
                    let (x, y) = self.tile;
                    self.rec(1, "m+0x0", &[x, y]);
                }
                fn road_changed(&mut self, flag: bool) -> i32 {
                    let (x, y) = self.tile;
                    self.rec(2, "c1+0x0", &[x, y, 1, 0, i32::from(flag)]);
                    self.a[3]
                }
                fn get_road_data(&mut self) -> bool {
                    let (x, y) = self.tile;
                    self.rec(3, "c1+0x0", &[x, y, 0, 0, 0]);
                    let k = self.reads;
                    self.reads += 1;
                    let null = self.a[16 + k.min(23)] != 0;
                    self.elem = if null { None } else { Some(4 + k.min(23)) };
                    wr32(
                        self.m,
                        0x14c,
                        if null {
                            0
                        } else {
                            // The window's own address: the port cannot see it, so the test holds this word
                            // to "non-zero" and nothing else.
                            1
                        },
                    );
                    !null
                }
                fn mark_and_trim(&mut self, list: i32) {
                    self.rec(4, "m+0x0", &[list]);
                }
                fn set_diags(&mut self, list: i32, param: i32) -> i32 {
                    self.rec(5, "m+0x0", &[list, param]);
                    self.queue.count += 1;
                    self.a[6]
                }
                fn mark_splits(&mut self) {
                    self.rec(6, "m+0x0", &[1]);
                }
                fn mark_white_lines(&mut self) {
                    self.rec(7, "m+0x0", &[1]);
                }
                fn mark_yellow_lines(&mut self) {
                    self.rec(8, "m+0x0", &[1]);
                }
                fn mark_intersections(&mut self) {
                    // Its argument is the `ecx` the call before left, which the stub leaves as `this`.
                    self.log.push("s9:m+0x0,m+0x0".into());
                }
                fn fixup_lines(&mut self, arg: i32) {
                    self.rec(10, "m+0x0", &[arg]);
                }
                fn elem_flags(&self) -> u32 {
                    rd32(&self.p[self.elem.expect("an element")], 4) as u32
                }
                fn set_elem_flags(&mut self, flags: u32) {
                    let k = self.elem.expect("an element");
                    wr32(&mut self.p[k], 4, flags as i32);
                }
                fn find_piece(&mut self) {
                    self.rec(11, "m+0x0", &[]);
                }
                fn redo_len(&self, which: usize) -> i32 {
                    self.redo[which]
                }
                fn redo_changed_roads(&mut self, which: usize, arg: i32, param: i32) {
                    let list = if which == 0 { "m+0x150" } else { "m+0x16c" };
                    self.rec(12, "m+0x0", &[]);
                    let t = self.log.last_mut().expect("just pushed");
                    t.push_str(&format!(",{list},{arg},{param}"));
                    // The stub raises the second list's count by one each call.
                    self.redo[1] += 1;
                }
                fn redo_clear(&mut self, which: usize) {
                    self.redo[which] = 0;
                }
                fn tile_flags(&self, _x: i32, _y: i32) -> u16 {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn cell_has_tiles(&self, _cx: i32, _cy: i32) -> bool {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn cell_flags(&self, _cx: i32, _cy: i32) -> u16 {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn set_cell_flags(&mut self, _cx: i32, _cy: i32, _bits: u16) {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn tile_rec(&self, _x: i32, _y: i32) -> Option<TileRec> {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn add_ambience(&mut self, _x: i32, _y: i32) {
                    unreachable!("the queue slice keeps param at 0")
                }
                fn finish(&mut self) {
                    self.redo = [0, 0];
                    self.queue.count = 0;
                }
            }
            let queue = load(
                &p[0],
                rd32(m, 0xe4),
                rd32(m, 0xe8),
                i16::from_le_bytes([m[0xec], m[0xed]]),
                rd32(m, 0xf8),
                m[0xf4],
            );
            let remove = load(
                &p[1],
                rd32(&c[3], 4),
                rd32(&c[3], 8),
                i16::from_le_bytes([c[3][0xc], c[3][0xd]]),
                rd32(&c[4], 4),
                c[4][0],
            );
            let redo = [rd32(m, 0x154), rd32(m, 0x170)];
            let mut v = V {
                a,
                m,
                c,
                p,
                log: Vec::new(),
                queue,
                remove,
                tile: (0, 0),
                elem: None,
                reads: 0,
                redo,
            };
            add_roads_steps(&mut v, a[0]);
            log.extend(std::mem::take(&mut v.log));
            let (queue, remove, redo) = (v.queue.clone(), v.remove.clone(), v.redo);
            let (m, c, p) = (v.m, v.c, v.p);
            store(&queue, &mut p[0]);
            store(&remove, &mut p[1]);
            wr32(m, 0xe4, queue.count);
            m[0xf4] = queue.flags;
            wr32(m, 0xf8, queue.cursor);
            wr32(m, 0x154, redo[0]);
            wr32(m, 0x170, redo[1]);
            wr32(&mut c[3], 4, remove.count);
            c[4][0] = remove.flags;
            wr32(&mut c[4], 4, remove.cursor);
            if log.is_empty() {
                log.push("-".into());
            }
            // The element pointer is an address in the original; the port holds it to null or not.
            let want_m = unhex(after.trim().split(';').next().expect("this"));
            let w = rd32(&want_m, 0x14c);
            assert_eq!(w != 0, rd32(m, 0x14c) != 0, "add_roads — the element pointer's nullness, row `{line}`");
            wr32(m, 0x14c, w);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "add_roads — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_roads — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_roads — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "add_roads — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Roads::add_roads@0088f4b0: {n} rows agree");
}
