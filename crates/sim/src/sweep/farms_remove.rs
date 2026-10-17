//! `Farms::remove@008d8e50` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Farms::remove@008d8e50`, run by `tools/emu/sweep_farms_remove.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0..p10 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; 4 random bytes at 0x00c0a908; vtable slot +0xb0 (0 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x008e3da0 (2 args) stubbed; the word at `0xc0a914` points at 576 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p2+0` points at 64 random bytes less `0` (placed); the word at `p2+4` points at 64 random bytes less `0` (placed); the word at `p3+0x18` points at 576 random bytes less `0` (placed); the word at `p4+0x18` points at 576 random bytes less `0` (placed); the word at `p5+0x18` points at 576 random bytes less `0` (placed); the word at `p6+0x18` points at 576 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 prints the 16 bytes its argument 1 points at, hex, after the record (`/` a call); stub 1 writes a value in -3..9 through its argument 0, printed after the answers (`o` in a place); stub 1 writes a value in -3..9 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_remove.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` the farm index, below the count `c1`; `p[0]` the farms (0xc0 each: `+0` player, `+4` object
            // index, `+0xbc` cleared, bit 2 of `+0xbd` an ambience to remove). Its object is
            // `p[3 + 2 * player + index]` and that object's type `p[7 + 2 * player + index]`. Stub 1
            // (`tile_corner`, ecx the slot-0xb0 stub's answer `a[1]`) writes the corner `(a[4], a[5])`;
            // stub 2 is handed the type and a rectangle of the corner and the type's size (`+0x234`,
            // `+0x238`), tiles of 0xc0, which the row prints.
            let mut tokens: Vec<String> = Vec::new();
            if a[0] < rd(&c[1], 0) {
                let at = a[0] as usize * 0xc0;
                p[0][at + 0xbc] = 0;
                if p[0][at + 0xbd] & 4 != 0 {
                    let (player, index) = (rd(&p[0], at) as usize, rd(&p[0], at + 4) as usize);
                    let (obj, ty) = (3 + 2 * player + index, 7 + 2 * player + index);
                    let (x, y) = (a[4], a[5]);
                    let (tw, th) = (rd(&p[ty], 0x234), rd(&p[ty], 0x238));
                    let rect = [
                        x * 0xc0,
                        x * 0xc0 + tw * 0xc0,
                        y * 0xc0,
                        y * 0xc0 + th * 0xc0,
                    ];
                    tokens.push(format!("s0:p{obj}+0x0"));
                    tokens.push(format!("s1:{},sp-0x14,sp-0x4", a[1]));
                    tokens.push(format!("s2:p{ty}+0x0,p{ty}+0x0,sp-0x24"));
                    tokens.push(
                        rect.iter()
                            .map(|v| {
                                v.to_le_bytes()
                                    .iter()
                                    .map(|b| format!("{b:02x}"))
                                    .collect::<String>()
                            })
                            .collect(),
                    );
                }
            }
            if tokens.is_empty() {
                tokens = vec!["-".to_string(); 2];
            }
            log = tokens;
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "remove — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "remove — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "remove — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Farms::remove@008d8e50: {n} rows agree");
}
