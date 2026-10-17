//! `MountainsData::total_gather_size@0089c110` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `MountainsData::total_gather_size@0089c110`, run by `tools/emu/sweep_mountains_data_total_gather_size.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10..p11 the stubs' answer window(s); callee 0x0089d080 (2 args) stubbed; callee 0x0089cf90 (2 args) stubbed; callee 0x0089d170 (4 args) stubbed; the word at `m+0xcc` points at 16 random bytes less `0` (placed); the word at `m+0x5c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); stub 0 answers the address of 8 random bytes, a `p` window after the places; stub 1 answers the address of 8 random bytes, a `p` window after the places; stub 2 answers 4 values in call order, printed after the answers; stub 2 answers -5000..5000 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_mountains_data_total_gather_size.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]`, `a[1]` are the addresses of the two answer windows `p[10]`, `p[11]`
            // (`mnt_loc_y`, `mnt_loc_x`: the value is the word at the pointer they answer),
            // `a[3..7]` the four `gather_size` answers in call order. Each of the
            // `+0xa4` mountains names a range type (`p[0]`, the `+0xcc` array), whose
            // record `p[2 + type]` (`+0x5c` array `p[1]`) is `gather_size`'s object (`ecx`
            // the record + 4 + the `+0xc` of its `+4`, held at 0) handed (x, y, -1, 0).
            let count = rd(m, 0xa4);
            let mut calls: Vec<String> = Vec::new();
            let mut sum = 0i32;
            for i in 0..count.max(0) as usize {
                let t = rd(&p[0], 4 * i) as usize;
                calls.push(format!("p{}+0x4,{},{},-1,0", 2 + t, rd(&p[11], 0), rd(&p[10], 0)));
                sum = sum.wrapping_add(a[3 + i]);
            }
            log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
            i64::from(sum)
        };
        assert_eq!(got, want_eax, "total_gather_size — eax, row `{line}`");
        assert_eq!(log, want_log, "total_gather_size — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "total_gather_size — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "total_gather_size — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "total_gather_size — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("MountainsData::total_gather_size@0089c110: {n} rows agree");
}
