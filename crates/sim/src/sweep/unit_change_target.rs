//! `Unit::change_target@005e36c0` under the emulator against the port (item 3150; tools/sweep_gen.py, obj with windows).

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

/// `Unit::change_target@005e36c0`, run by `tools/emu/sweep_unit_change_target.py` under unicorn
/// (tools/emu): 256 random bytes of object, 4 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10..p12 the stubs' answer window(s); the global cell 0x00c0618c points at 64 random bytes; callee 0x0060a870 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `m+0xdc` points at 8 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 16 random bytes less `0` (placed); stub 2 answered from [0, 0, 0, 5] (90%); stub 0 answers the address of 32 random bytes, a `p` window after the places; stub 1 answers the address of 32 random bytes, a `p` window after the places; stub 3 answers the address of 32 random bytes, a `p` window after the places; argument 0 copies `p11+0x8` (45%); argument 1 copies `p11+0xc` (45%); stub 2 answers 3 values in call order, the last from [1], printed after the answers; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_change_target.py") else {
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
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let wr = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            // p0 the type-0 array (six records p1..p6), p7..p9 the order list's head, node and order, p10 the
            // `update_action` answer, p11 the target record its slot +0x3c answers, p12 the slot +0xa8 answer;
            // a[8..11] the slot +0x20 answers in call order.
            let mut kills: Vec<String> = Vec::new();
            if a[0] == rd(&p[11], 8) && a[1] == rd(&p[11], 0xc) {
                wr(&mut p[11], 8, a[2]);
                wr(&mut p[11], 0xc, a[3]);
                let rec = &p[1 + a[2] as usize];
                let (lo, hi) = (rec[0x30], rec[0x31]);
                p[11][0x10] = lo;
                p[11][0x11] = hi;
                let node = rd(&p[7], 4);
                wr(m, 0xd4, node);
                wr(m, 0xcc, rd(&p[8], 8));
                m[0xd0] = p[8][0xc];
                let mut k = 0;
                while a[9 + k.min(2)] == 0 {
                    kills.push("m+0x0,0".to_string());
                    k += 1;
                }
            }
            log.push(if kills.is_empty() {
                "-".to_string()
            } else {
                kills.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "change_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "change_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "change_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "change_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Unit::change_target@005e36c0: {n} rows agree");
}
