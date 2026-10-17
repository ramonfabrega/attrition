//! `Mountains::walk_data@0089d320` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `Mountains::walk_data@0089d320`, run by `tools/emu/sweep_mountains_walk_data.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00e85f64 points at 32 random bytes; argument 0 names 16 random bytes; vtable slot +0x4 (1 args) stubbed; callee 0x0047c660 (2 args) stubbed; callee 0x004a46d0 (2 args) stubbed; callee 0x00473120 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 leaves `ecx` as a callee would, a value from [1, 2, 3, 4096], after the other numbers on the row; stub 1 leaves `ecx` as a callee would, a value from [5, 6, 7, 8192], after the other numbers on the row; stub 2 leaves `ecx` as a callee would, a value from [9, 10, 11, 12288], after the other numbers on the row; stub 3 leaves `ecx` as a callee would, a value from [13, 14, 15, 16384], after the other numbers on the row; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_mountains_walk_data.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The walker `p0` is handed its slot 4 (`ecx` the walker) the string-table entry
            // `[c0+0x10] + 0x18178`; then the `mountains` record `[c1+8]` (the global
            // object's base) hands four arrays to the walker: `SimpleArray::walk_data` at
            // `+0xe85fcc`, `+0xe85fe8` (one callee), `+0xe86004`, `+0xe86020`, each given
            // `(walker, ecx)` — the `ecx` the previous call left, which the stubs set
            // (`a[5..9]`, the walker's slot first).
            let base = rd(&c[1], 8);
            log.push(format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0x18178)));
            log.push(format!(
                "{},p0+0x0,{}/{},p0+0x0,{}",
                base.wrapping_add(0xe85fcc),
                a[5],
                base.wrapping_add(0xe85fe8),
                a[6]
            ));
            log.push(format!("{},p0+0x0,{}", base.wrapping_add(0xe86004), a[6]));
            log.push(format!("{},p0+0x0,{}", base.wrapping_add(0xe86020), a[7]));
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "walk_data — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Mountains::walk_data@0089d320: {n} rows agree");
}
