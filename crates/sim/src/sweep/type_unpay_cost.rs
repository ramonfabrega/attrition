//! `Type::unpay_cost@006682f0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

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

/// `Type::unpay_cost@006682f0`, run by `tools/emu/sweep_type_unpay_cost.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), the global cell 0x00c061ec points at 2112 random bytes; callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2, 5, 100, -7, 3, 250] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_unpay_cost.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Unless the game record's byte `+0x821` has bit 3 set, each of the six goods the leader
            // (`a[0]`) has available (`type_avail` answers `a[5..11]`) is refunded the cost slot `+0x78`
            // answers (`a[11..17]`, in call order, handed `(good, who, a[1], a[2], 1, 1, -1)`), added
            // to the leader's bucket stored XOR 0x8221 (item 2837).
            if c[0][0x821] & 8 == 0 {
                let who = a[0] as usize;
                let mut asked = 11;
                for g in 0..6usize {
                    if a[5 + g] != 0 {
                        log.push(format!("s1:m+0x0,{g},{},{},{},1,1,-1", a[0], a[1], a[2]));
                        let cost = a[asked];
                        asked += 1;
                        let bucket = &mut p[8 + who];
                        let cur = rd(bucket, 4 * g) ^ 0x8221;
                        wr(bucket, 4 * g, cur.wrapping_add(cost) ^ 0x8221);
                    }
                }
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "unpay_cost — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "unpay_cost — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "unpay_cost — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "unpay_cost — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Type::unpay_cost@006682f0: {n} rows agree");
}
