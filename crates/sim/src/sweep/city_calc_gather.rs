//! `City::calc_gather@00737c60` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `City::calc_gather@00737c60`, run by `tools/emu/sweep_city_calc_gather.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 2 argument(s), the global cell 0x00c061e4 points at 1024 random bytes; the global cell 0x00c096f4 points at 64 random bytes; callee 0x00736890 (0 args) stubbed; callee 0x00736820 (0 args) stubbed; callee 0x0066fcf0 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x007367a0 (0 args) stubbed; callee 0x006d5530 (5 args) stubbed; stub 2 answered from [-1, -2, 0, 425, 423, 7] (90%); stub 3 answered from [0, 1, 2] (90%); stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 0 answers 0..5 every row; stub 1 answers 0..5 every row; stub 4 answers 0..5 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_calc_gather.py") else {
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
            // `a` is the two arguments, the six stubs' answers
            // (`granary_level`, `lumber_level`, `get_enhancer`,
            // `count_buildings`, `smelter_level`, `calc_city_resources`),
            // then `get_enhancer`'s two in call order and
            // `count_buildings`' two. `c0` is the constants (the bonus
            // tables at `0x2b8 + 4·level` and on), `c1` the good-type list
            // (`+0x10` metal, `+0x14` oil, handed as `ecx`).
            let (mut enh, mut cnt) = (0usize, 0usize);
            // The `ecx` the last callee left: `calc_city_resources` is
            // pushed it in `param_2`'s place (737d1e), and a stub keeps it.
            let mut left = String::new();
            let flags = i32::from(u16::from_le_bytes([m[4], m[5]]));
            let bytes = crate::economy::calc_gather_steps(
                &mut (&mut log, &mut left),
                flags,
                |_| a[2],
                |_| a[3],
                |(log, left), r| {
                    let off = if r == crate::economy::Resource::Metal {
                        0x10
                    } else {
                        0x14
                    };
                    let ecx = rd(&c[1], off).to_string();
                    log.push(format!("s2:{ecx}"));
                    **left = ecx;
                    let v = a[8 + enh];
                    enh += 1;
                    v
                },
                |(log, left), t| {
                    log.push(format!("s3:m+0x0,{t},0,1"));
                    **left = "m+0x0".into();
                    let v = a[10 + cnt];
                    cnt += 1;
                    v
                },
                |_| a[6],
                |_, r, lvl| {
                    let base = match r {
                        crate::economy::Resource::Food => 0x2b8,
                        crate::economy::Resource::Timber => 0x2cc,
                        _ => 0x2e0,
                    };
                    rd(&c[0], (base + 4 * lvl) as usize)
                },
                |(log, left), _| {
                    let who = i32::from(m[0x5e] as i8);
                    log.push(format!(
                        "s5:{},{},{},{},{},{}",
                        0xe3a390 + who * 0x6eec,
                        a[0],
                        left,
                        rd(m, 0xc),
                        rd(m, 0x10),
                        i16::from_le_bytes([m[6], m[7]])
                    ));
                },
            );
            m[0x56..0x5a].copy_from_slice(&bytes);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "calc_gather — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "calc_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "calc_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "calc_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("City::calc_gather@00737c60: {n} rows agree");
}
