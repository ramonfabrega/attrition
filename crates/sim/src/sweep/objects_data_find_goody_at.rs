//! `ObjectsData::find_goody_at@0065b7c0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_goody_at@0065b7c0`, run by `tools/emu/sweep_objects_data_find_goody_at.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), callee 0x0065c040 (1 args) stubbed; the word at `0xcae5fc` points at 64 random bytes less `0` (placed); the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p1+0x134` points at 448 random bytes less `0` (placed); stub 0 answered from [0, 1, 5, -7] (90%); stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_goody_at.py") else {
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
            // Two handles (`>> 8`, through the table at `0xcae5fc`) name a tile (x, y) of the map at `0xc061d0`
            // (width at +0, 28-byte tiles at the pointer in +0x134): a tile without the 0x100 flag whose byte
            // +2 is 1 or 2 answers -1; any other goes to the overload at `0x65c040`, handed the y handle.
            let hx = rd(&p[0], 4 * (a[0] >> 8) as usize);
            let hy = rd(&p[0], 4 * (a[1] >> 8) as usize);
            let at = (rd(&p[1], 0).wrapping_mul(hy).wrapping_add(hx) as usize) * 28;
            let tile = &p[2][at..at + 28];
            let flagged = u16::from_le_bytes([tile[0], tile[1]]) & 0x100 != 0;
            if !flagged && matches!(tile[2] as i8, 1 | 2) {
                log.push("-".into());
                -1
            } else {
                log.push(hy.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect());
                i64::from(a[3])
            }
        };
        assert_eq!(got, want_eax, "find_goody_at — eax, row `{line}`");
        assert_eq!(log, want_log, "find_goody_at — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_goody_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_goody_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_goody_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectsData::find_goody_at@0065b7c0: {n} rows agree");
}
