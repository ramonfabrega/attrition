//! `BuildType::place_roads@0063c580` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

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

// hand: view
use crate::place_roads_view::{self, PlaceRoadsView};

/// The row's windows as the view. `m` is the `BuildType`, `c0` the world record (`+0x18` width, `+0x1c`
/// height, `+0x138` the tile masks, the window `p2`), `c1` the objects cell (the windows `p3`/`p4` hold
/// each player's pointer array, `p5 + 2 * player + idx` the object), `c2` the `div_3_table`, `c3` the
/// function's `road` static (`+4` the list, the window `p15`; `+0xc` its length), `p9`/`p10` the cities'
/// pointer arrays (`p11 + 2 * player + city` a city record, its centre building the short at `+8`) and
/// `p16` the record slot `0xac` answers. The answers are `a[6..14]`, stubs 0 to 7, and `a[14]` is the
/// flag that laid the base `is_city` in slot `0x64`.
struct Emu<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: Vec<String>,
    xs: Vec<String>,
    ys: Vec<String>,
}

impl Emu<'_> {
    fn image_vtable(&self) -> bool {
        rd(self.m, 0) as u32 == 0xb42b94
    }
    fn object(&self, player: i32, idx: i32) -> String {
        format!("p{}+0x0", 5 + 2 * player + idx)
    }
}

impl PlaceRoadsView for Emu<'_> {
    fn road_clear(&mut self) {
        wr(&mut self.c[3], 12, 0);
    }
    fn is_city(&mut self) -> bool {
        // A row that laid the image vtable (`m+0`) copied the stub slots there before the base flag was
        // drawn, so slot `0x64` is the stub on those rows whatever the flag says.
        if self.a[14] != 0 && !self.image_vtable() {
            self.log.push("s1:m+0x0,414,0".to_string());
            self.a[7] != 0
        } else {
            self.log.push("s0:m+0x0".to_string());
            self.a[6] != 0
        }
    }
    fn connects_to_roads(&mut self) -> bool {
        self.log.push("s5:m+0x0".to_string());
        self.a[11] != 0
    }
    fn is_gatherer(&mut self) -> bool {
        if self.image_vtable() {
            rd(self.m, 0x2c0) & 0x40 != 0
        } else {
            self.log.push("s2:m+0x0".to_string());
            self.a[8] != 0
        }
    }
    fn size(&self) -> (i32, i32) {
        (rd(self.m, 0x234), rd(self.m, 0x238))
    }
    fn div3(&self, i: i32) -> i32 {
        rd(&self.c[2], 4 * usize::try_from(i).expect("a table index"))
    }
    fn world_dims(&self) -> (i32, i32) {
        (rd(&self.c[0], 0x18), rd(&self.c[0], 0x1c))
    }
    fn tile_mask(&self, x: i32, y: i32) -> u16 {
        let at = 2 * (rd(&self.c[0], 0x18) * y + x) as usize;
        u16::from_le_bytes([self.p[2][at], self.p[2][at + 1]])
    }
    fn set_road_at(&mut self, x: i32, y: i32, set: i32) {
        self.log.push(format!("s6:c0+0x0,sp,sp,{set},0,0"));
        self.xs.push(format!("{:08x}", (x as u32).swap_bytes()));
        self.ys.push(format!("{:08x}", (y as u32).swap_bytes()));
    }
    fn obj_can_road(&mut self, player: i32, idx: i32) -> bool {
        let t = format!("s3:{}", self.object(player, idx));
        self.log.push(t);
        self.a[9] != 0
    }
    fn obj_city(&mut self, player: i32, idx: i32) -> i16 {
        let t = format!("s4:{}", self.object(player, idx));
        self.log.push(t);
        i16::from_le_bytes([self.p[16][0x72], self.p[16][0x73]])
    }
    fn obj_mark_regen(&mut self, player: i32, idx: i32) {
        let t = format!("s4:{}", self.object(player, idx));
        self.log.push(t);
        self.p[16][0x61] |= 0x01;
    }
    fn city_building(&mut self, player: i32, city: i16) -> i16 {
        assert!((0..2).contains(&city), "a placed city");
        let w = &self.p[11 + 2 * player as usize + city as usize];
        i16::from_le_bytes([w[8], w[9]])
    }
    fn find_road(&mut self, city: i16, idx: i32, player: i32, building: i16) {
        self.log.push(format!(
            "s7:{city},c3+0x4,{idx},{player},{building},{player},-1,-1"
        ));
        wr(&mut self.c[3], 12, 5);
    }
    fn road_length(&self) -> i32 {
        rd(&self.c[3], 12)
    }
    fn set_road_length(&mut self, n: i32) {
        wr(&mut self.c[3], 12, n);
    }
    fn road_entry(&self, i: i32) -> (i32, i32) {
        let at = 16 * i as usize;
        (rd(&self.p[15], at), rd(&self.p[15], at + 4))
    }
}
// end hand

/// `BuildType::place_roads@0063c580`, run by `tools/emu/sweep_build_type_place_roads.py` under unicorn
/// (tools/emu): 712 random bytes of object, 6 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p16 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 32 random bytes at 0x00ee14e8; vtable slot +0x64 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006398c0 (0 args) stubbed; callee 0x006b43b0 (5 args) stubbed; callee 0x00688a40 (7 args) stubbed; the word at `c0+0x138` points at 1024 random bytes less `0` (placed); the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `c1+0x30` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `p4+0` points at 64 random bytes less `0` (placed); the word at `p4+4` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p9+4` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `c3+4` points at 256 random bytes less `0` (placed); stub 4's answer points at 128 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [1, 1, 0] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0] (90%); stub 7 answered from [0] (90%); the word at `m+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `fs:[0x2c]` -> a TLS array -> one block whose word +0x8 is 0 (a `_TSS` guard read as initialised); stub 7 writes 5 (4 bytes) at `c3+0xc` each call; 2 rows 0x1c apart at `@0xc09970`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_place_roads.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // hand: want
        let want_log: Vec<String> = calls
            .split_whitespace()
            .map(|t| {
                // `set_road_at`'s first two arguments are the addresses of the function's own locals,
                // named by their depth below the entry `esp`; the coordinates ride the deref columns.
                t.split(',')
                    .map(|f| if f.starts_with("sp-") { "sp" } else { f })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect();
        // end hand
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String>;
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            let mut emu = Emu {
                a,
                m,
                c,
                p,
                log: Vec::new(),
                xs: Vec::new(),
                ys: Vec::new(),
            };
            place_roads_view::place_roads_steps(&mut emu, a[0], a[1], a[2], a[3], a[4], a[5]);
            let group = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            let (xs, ys) = (group(&emu.xs), group(&emu.ys));
            log = emu.log;
            log.push(xs);
            log.push(ys);
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "place_roads — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "place_roads — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "place_roads — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "place_roads — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("BuildType::place_roads@0063c580: {n} rows agree");
}
