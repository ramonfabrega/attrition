//! `PathFinder::find_upath_restore@00688f40` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::find_upath_restore@00688f40`, run by `tools/emu/sweep_path_finder_find_upath_restore.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 3 argument(s), the global cell 0x00c061bc points at 16 random bytes; the global cell 0x00c0618c points at 144 random bytes; 8 random bytes at 0x00e85ec0; callee 0x00682f30 (6 args) stubbed; the word at `c1+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[2]` points at 24 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_find_upath_restore.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            { let x = rd(&p[1], 0x10) ^ 0x63637; let y = rd(&p[1], 0x14) ^ 0x63637; let sq = rd(&c[0], 4 * a[1] as usize).wrapping_mul(rd(&c[0], 4 * a[1] as usize)).max(1); wr(&mut c[2], 0, 300 / sq); log.push(format!("{},{},{},{},{},{},0", y, a[0], x, y, a[1], a[2])); wr(&mut c[2], 4, 0); i64::from(a[3]) }
        };
        assert_eq!(got, want_eax, "find_upath_restore — eax, row `{line}`");
        assert_eq!(log, want_log, "find_upath_restore — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_upath_restore — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_upath_restore — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "find_upath_restore — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PathFinder::find_upath_restore@00688f40: {n} rows agree");
}
