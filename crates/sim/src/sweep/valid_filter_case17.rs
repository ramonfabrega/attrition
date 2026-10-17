//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

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

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_valid_filter_case17.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), vtable slot +0x3c (0 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x00739560 (1 args) stubbed; callee 0x007395c0 (2 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p27+0` points at 48 random bytes less `0` (placed); the word at `p27+4` points at 48 random bytes less `0` (placed); the word at `p27+8` points at 48 random bytes less `0` (placed); the word at `p27+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p32+0` points at 48 random bytes less `0` (placed); the word at `p32+4` points at 48 random bytes less `0` (placed); the word at `p32+8` points at 48 random bytes less `0` (placed); the word at `p32+12` points at 48 random bytes less `0` (placed); the word at `p18+0` points at 12 random bytes less `0` (placed); the word at `p19+0` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 12 random bytes less `0` (placed); the word at `p21+0` points at 12 random bytes less `0` (placed); the word at `p22+0` points at 12 random bytes less `0` (placed); the word at `p23+0` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 12 random bytes less `0` (placed); the word at `p25+0` points at 12 random bytes less `0` (placed); the word at `p26+0` points at 12 random bytes less `0` (placed); the word at `p37+0` points at 8 random bytes less `0` (placed); the word at `p37+4` points at 8 random bytes less `0` (placed); the word at `p37+8` points at 8 random bytes less `0` (placed); the word at `p38+0` points at 8 random bytes less `0` (placed); the word at `p38+4` points at 8 random bytes less `0` (placed); the word at `p38+8` points at 8 random bytes less `0` (placed); the word at `p39+0` points at 8 random bytes less `0` (placed); the word at `p39+4` points at 8 random bytes less `0` (placed); the word at `p39+8` points at 8 random bytes less `0` (placed); the word at `p40+0` points at 8 random bytes less `0` (placed); the word at `p40+4` points at 8 random bytes less `0` (placed); the word at `p40+8` points at 8 random bytes less `0` (placed); the word at `p41+0` points at 8 random bytes less `0` (placed); the word at `p41+4` points at 8 random bytes less `0` (placed); the word at `p41+8` points at 8 random bytes less `0` (placed); the word at `p42+0` points at 8 random bytes less `0` (placed); the word at `p42+4` points at 8 random bytes less `0` (placed); the word at `p42+8` points at 8 random bytes less `0` (placed); the word at `p43+0` points at 8 random bytes less `0` (placed); the word at `p43+4` points at 8 random bytes less `0` (placed); the word at `p43+8` points at 8 random bytes less `0` (placed); the word at `p44+0` points at 8 random bytes less `0` (placed); the word at `p44+4` points at 8 random bytes less `0` (placed); the word at `p44+8` points at 8 random bytes less `0` (placed); the word at `p45+0` points at 8 random bytes less `0` (placed); the word at `p45+4` points at 8 random bytes less `0` (placed); the word at `p45+8` points at 8 random bytes less `0` (placed); stub 0 answered from [0] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1, 1] (90%); stub 0 answers the address of 128 random bytes, a `p` window after the places; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 27 values in call order, printed after the answers; stub 4 answers 54 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 9 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x408:0x4 (`p` windows after the pointed buffers, before the places); 9 rows 0x1c apart at `0xc061d4`, slices +0x10:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_valid_filter_case17.py") else {
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
            {
                // Arm 17: the object `objects[who][p0]` must carry flag 0x20 (+8); slot +0x3c answers the placed p73
                // whose signed word at +0x72 is `s`, not negative. Then each valid player b (bit 0 of its row's first
                // word) other than `who` must pass `0x6edb50(b)` and `0x6db810(0x2ac)` (stubs 1 and 2, on `who`'s
                // row); each entry j of b's list (count at +0x408), but `s` of `who`'s own, whose +4 bit 0 is set,
                // must pass `0x739560(who)` (stub 3), `0x7395c0(j, b)` on `who`'s entry s, and `0x7395c0(s, who)`
                // (stub 4, answered in call order) on b's entry j, which returns 1. Else 0, after the eight players (the loop ends when the offset 0x10 + 0x1c b reaches 0xf0).
                let who = a[a.len() - 1] as usize;
                let i0 = a[0] as usize;
                let mut logs: Vec<Vec<String>> = vec![Vec::new(); 5];
                let mut seqs = [9usize, 17, 25, 52];
                let mut next = |k: usize| {
                    let v = a[seqs[k]];
                    seqs[k] += 1;
                    v
                };
                let obj = |b: usize, j: usize| 46 + 3 * b + j;
                let out = 'arm: {
                    if rd(&p[28 + 5 * who + i0], 8) & 0x20 == 0 {
                        break 'arm 0;
                    }
                    logs[0].push(format!("p{}+0x0", 28 + 5 * who + i0));
                    let s = i32::from((rd(&p[73], 0x70) >> 16) as i16);
                    if s < 0 {
                        break 'arm 0;
                    }
                    for b in 0..8usize {
                        if rd(&p[2 * b], 0) & 1 == 0 {
                            continue;
                        }
                        if b != who {
                            logs[1].push(format!("p{}+0x0,{b}", 2 * who));
                            if next(0) == 0 {
                                continue;
                            }
                            logs[2].push(format!("p{}+0x0,684", 2 * who));
                            if next(1) == 0 {
                                continue;
                            }
                        }
                        let count = rd(&p[2 * b + 1], 0);
                        for j in 0..count.max(0) {
                            if j == s && who == b {
                                continue;
                            }
                            let q = obj(b, j as usize);
                            if rd(&p[q], 4) & 1 == 0 {
                                continue;
                            }
                            logs[3].push(format!("p{q}+0x0,{who}"));
                            if next(2) == 0 {
                                continue;
                            }
                            logs[4].push(format!("p{}+0x0,{j},{b}", obj(who, s as usize)));
                            if next(3) == 0 {
                                continue;
                            }
                            logs[4].push(format!("p{q}+0x0,{s},{who}"));
                            if next(3) != 0 {
                                break 'arm 1;
                            }
                        }
                    }
                    0
                };
                for l in logs {
                    log.push(if l.is_empty() { "-".into() } else { l.join("/") });
                }
                out
            }
        };
        assert_eq!(got, want_eax, "valid_filter — eax, row `{line}`");
        assert_eq!(log, want_log, "valid_filter — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_filter — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "valid_filter — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "valid_filter — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Search::valid_filter@0067dbb0: {n} rows agree");
}
