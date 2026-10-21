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

/// `Unit::detect_unit_collision@00617060`, run by `tools/emu/sweep_unit_detect_unit_collision_gates.py` under unicorn
/// (tools/emu): 256 random bytes of object, 7 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c06188 points at 8 random bytes; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x10c (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; callee 0x005fa8b0 (3 args) stubbed; callee 0x00682540 (8 args) stubbed; callee 0x00609fa0 (4 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x004613d0 (4 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xb8` points at 64 random bytes less `0` (placed); slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answers 11..15 every row; stub 7 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 4 writes a value in 0..40 through its argument 5, printed after the answers (`o` in a place); stub 4 writes a value in 0..40 through its argument 6, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_detect_unit_collision_gates.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // **The port is `detect_collision_view::detect_unit_collision_steps`**, the walk
            // `Sim::detect_unit_collision` runs over its `SimDetect`; here the view is the emulated
            // original's windows. `a[..7]` the arguments (x, y, quick, boats, the unused fifth,
            // nocoll, top_only), `a[7..15]` the eight stubs' answers (0 type vslot `0x10c`, 1
            // `is_hero`, 2 `is_supply`, 3 `detect_boat_collision`, 4 `collide_here`, 5
            // `will_be_corner`, 6 `action_type`, 7 `get_down`), `a[15..17]` `collide_here`'s
            // two out-words (hit x, y), `a[17..19]` the flags that lay the image's `is_hero`
            // and `is_supply` in the vtable (the inline arms). Windows: `m` the unit, `c0` the
            // world (`+0` width, `+4` height), `c1` the game (`+0x550` the frame), `c2` the
            // `div_3_table`, `p0` the type record, `p1` the path's four nodes. This slice's
            // `get_down` always answers "nothing" and `action_type` is never the attack, so
            // the chain walk and the overreach are the other slice's.
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
                cxs: Vec::new(),
                cys: Vec::new(),
            };
            let (eax, _) = detect_unit_collision_steps(&mut view, args);
            let (cxs, cys) = (view.cxs, view.cys);
            if log.is_empty() {
                log.push("-".to_string());
            }
            let col = |v: &[i32]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.iter()
                        .map(|x| {
                            x.to_le_bytes()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("/")
                }
            };
            log.push(col(&cxs));
            log.push(col(&cys));
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
                &want[4 + i],
                "detect_unit_collision — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::detect_unit_collision@00617060: {n} rows agree");
}

// hand: view
use sim::detect_collision_view::{DetectCollisionView, unit_cell_of};

/// The row's windows as the view. `m` the unit, `c0` the world, `c1` the game, `c2` the
/// `div_3_table` (a word `i / 3` at index `i`), `p0` the type record, `p1` the path's nodes.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    cxs: Vec<i32>,
    cys: Vec<i32>,
}

impl DetectCollisionView for Rows<'_> {
    type Node = (i32, i32);

    fn domain(&mut self) -> i32 {
        rd(&self.p[0], 0x218)
    }
    fn type_is_siege(&mut self) -> bool {
        self.log.push("s0:p0+0x0".to_string());
        self.a[7] != 0
    }
    fn is_hero(&mut self) -> bool {
        if self.a[17] != 0 {
            return rd(&self.p[0], 0x2b8) & 0x20 != 0;
        }
        self.log.push("s1:m+0x0".to_string());
        self.a[8] != 0
    }
    fn is_supply(&mut self) -> bool {
        if self.a[18] != 0 {
            return rd(&self.p[0], 0x2b8) & 0x40 != 0;
        }
        self.log.push("s2:m+0x0".to_string());
        self.a[9] != 0
    }
    fn detect_boat_collision(&mut self, x: i32, y: i32) -> bool {
        self.log.push(format!("s3:m+0x0,{x},{y},1"));
        self.a[10] != 0
    }
    fn path_detour(&mut self) -> bool {
        let n = rd(self.m, 0xc0);
        if n == 0 {
            return false;
        }
        let at = if n >= 1 { (n as usize) * 16 - 16 } else { 0 };
        self.p[1][at + 0xc] & 8 != 0
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
        let o = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
        let who = i32::from(self.m[9]);
        let size = rd(&self.p[0], 0x248);
        self.log.push(format!(
            "s4:sp-0x34,{o},{who},{},{},{size},sp-0x34,sp-0x3c,{}",
            cell.0,
            cell.1,
            i32::from(nocoll)
        ));
        (self.a[11] != 0).then_some((self.a[15], self.a[16]))
    }
    fn will_be_corner(&mut self, hit: (i32, i32), cell: (i32, i32)) -> i32 {
        self.log.push(format!(
            "s5:m+0x0,{},{},{},{}",
            hit.0, hit.1, cell.0, cell.1
        ));
        self.a[12]
    }
    fn action_type(&mut self) -> i32 {
        self.log.push("s6:m+0x0".to_string());
        self.a[13]
    }
    fn attack_target_exists(&mut self) -> bool {
        unreachable!("the gates slice never attacks")
    }
    fn max_range(&mut self) -> i32 {
        unreachable!("the gates slice never attacks")
    }
    fn attack_dist(&mut self, _x: i32, _y: i32) -> i32 {
        unreachable!("the gates slice never attacks")
    }
    fn world_size(&mut self) -> (i32, i32) {
        (rd(&self.c[0], 0), rd(&self.c[0], 4))
    }
    fn get_down(&mut self, cell: (i32, i32)) -> Option<Self::Node> {
        self.log
            .push("s7:c0+0x0,sp-0x4,sp-0x14,sp-0x2c,sp-0x38".to_string());
        self.cxs.push(cell.0);
        self.cys.push(cell.1);
        (self.a[14] != 0).then_some((0, 0))
    }
    fn node_id(&mut self, n: Self::Node) -> (i32, i32) {
        n
    }
    fn node_next(&mut self, _n: Self::Node) -> Self::Node {
        unreachable!("the gates slice walks no chain")
    }
    fn node_is_me(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_alive(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_air(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_on_map(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_is_here(&mut self, _n: Self::Node, _hit: (i32, i32)) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_action_type(&mut self, _n: Self::Node) -> i32 {
        unreachable!("the gates slice walks no chain")
    }
    fn is_moving_me(&mut self) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_is_moving(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn guard_targets_me(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_same_owner(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn my_block(&mut self) -> i32 {
        unreachable!("the gates slice walks no chain")
    }
    fn node_block(&mut self, _n: Self::Node) -> i32 {
        unreachable!("the gates slice walks no chain")
    }
    fn groups_shared(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_searching(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_front_order(&mut self, _n: Self::Node) -> Option<i32> {
        unreachable!("the gates slice walks no chain")
    }
    fn node_cast_soft(&mut self, _n: Self::Node) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn node_corner(&mut self, _n: Self::Node, _hit: (i32, i32)) -> i32 {
        unreachable!("the gates slice walks no chain")
    }
    fn record_hit(&mut self, _n: Self::Node) {
        unreachable!("the gates slice walks no chain")
    }
    fn front_is_moving(&mut self) -> bool {
        unreachable!("the gates slice walks no chain")
    }
    fn set_move_x(&mut self, _x: i32) {
        unreachable!("the gates slice walks no chain")
    }
    fn set_move_y(&mut self, _y: i32) {
        unreachable!("the gates slice walks no chain")
    }
    fn set_half_step(&mut self) {
        unreachable!("the gates slice walks no chain")
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
