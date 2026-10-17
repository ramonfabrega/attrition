//! `Border::rebuild_visible_list@0087fd70` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `Border::rebuild_visible_list@0087fd70`, run by `tools/emu/sweep_border_rebuild_visible_list.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; callee 0x008642e0 (2 args) stubbed; callee 0x00422300 (1 args) stubbed; the word at `m+0x14` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); the word at `m+0x30` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_rebuild_visible_list.py") else {
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
            // The visible list (count +0x24, capacity +0x28, growth +0x2c, array pointer +0x30) is rebuilt
            // from the normal list (count +0x8, array +0x14 of record pointers; a record's rectangle is
            // the shorts at +0x18..+0x1e): the count drops to 0, then each record whose size passes the
            // visibility stub (cdecl; ecx = the record's x0, arguments the width then the height) is
            // appended, the list grown first (stub 1, ecx = this+0x20, the growth) when full.
            let sh = |r: &[u8], o: usize| i32::from(i16::from_le_bytes([r[o], r[o + 1]]));
            let (mut vis, mut grow): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            wr(m, 0x24, 0);
            let n = rd(m, 8).max(0) as usize;
            for i in 0..n {
                let r = &p[1 + i];
                let (x0, dx, dy) = (sh(r, 0x18), sh(r, 0x1a) - sh(r, 0x18), sh(r, 0x1e) - sh(r, 0x1c));
                vis.push(format!("{x0},{dx},{dy}"));
                if a[2 + i.min(7)] != 0 {
                    if rd(m, 0x24) >= rd(m, 0x28) {
                        grow.push(format!("m+0x20,{}", i32::from(i16::from_le_bytes([m[0x2c], m[0x2d]]))));
                    }
                    let cnt = rd(m, 0x24) as usize;
                    let ptr = rd(&p[0], i * 4);
                    wr(&mut p[9], cnt * 4, ptr);
                    wr(m, 0x24, cnt as i32 + 1);
                }
            }
            let join = |v: Vec<String>| if v.is_empty() { "-".to_string() } else { v.join("/") };
            log.push(join(vis));
            log.push(join(grow));
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "rebuild_visible_list — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "rebuild_visible_list — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "rebuild_visible_list — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "rebuild_visible_list — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Border::rebuild_visible_list@0087fd70: {n} rows agree");
}
