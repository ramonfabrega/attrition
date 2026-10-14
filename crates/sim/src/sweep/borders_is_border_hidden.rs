//! `Borders::is_border_hidden@00882300` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

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

/// `Borders::is_border_hidden@00882300`, run by `tools/emu/sweep_borders_is_border_hidden.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c06210 points at 672 random bytes; callee 0x006e1370 (1 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_is_border_hidden.py") else {
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
            {
    // Leader a0 asked has_tribe_bonus(0x13) (its row of the leaders the
    // cell 0xc061e0 holds); with it, the console's leader (+0x298 of the
    // console) names its own index (+8): a0 not it, and not at 2 both ways
    // in the diplos (+0x74, a word an index), answers 1.
    let who = a[0] as usize;
    let mut row = vec!["-".to_string(); 9];
    row[who] = "r+0x0,19".into();
    log.extend(row);
    let mut v = 0;
    if a[1] != 0 {
        let me = rd(&c[0], 0x298) as usize;
        let mine = rd(&p[2 * me], 0);
        if a[0] != mine
            && (rd(&p[2 * me + 1], 4 * who) != 2 || rd(&p[2 * who + 1], 4 * mine as usize) != 2)
        {
            v = 1;
        }
    }
    v
}
        };
        assert_eq!(got, want_eax, "is_border_hidden — eax, row `{line}`");
        assert_eq!(log, want_log, "is_border_hidden — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_border_hidden — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_border_hidden — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "is_border_hidden — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Borders::is_border_hidden@00882300: {n} rows agree");
}
