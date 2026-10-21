//! `Tribe::parse@006f0560` under the emulator against the port (item 3400; tools/sweep_gen.py, obj with windows).

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

/// `Tribe::parse@006f0560`, run by `tools/emu/sweep_tribe_parse.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), `p` windows in order: none; the global cell 0x00c061e4 points at 2272 random bytes; the global cell 0x00c8cd00 points at 64 random bytes; 40 random bytes at 0x00cc4b50; callee 0x00a1b830 (3 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cdd0 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
/// Tribe::parse's switch on `this.tribe` (`+0x54`): per tribe the `(parse_numbers index, Constants
/// byte offset, absolute value)` copies, read off the type record's field offsets; Russia (13) also
/// builds `parse_strings`. Inline port, a NAMESAKE of nothing: the crate has no tooltip-text loader.
const TABLE: [&[(usize, usize, bool)]; 24] = [
    &[
        (0, 0x56c, false),
        (1, 0x578, false),
        (2, 0x580, false),
        (3, 0x570, false),
    ],
    &[
        (0, 0x594, false),
        (1, 0x588, false),
        (2, 0x58c, false),
        (3, 0x590, false),
    ],
    &[(0, 0x598, false), (1, 0x5a0, false)],
    &[
        (1, 0x5a4, false),
        (2, 0x5a8, false),
        (3, 0x5b0, false),
        (4, 0x5b4, false),
        (5, 0x5b8, false),
    ],
    &[
        (0, 0x5bc, false),
        (1, 0x5c0, false),
        (2, 0x5c8, false),
        (3, 0x5cc, false),
        (4, 0x5d0, false),
    ],
    &[
        (0, 0x5d4, false),
        (1, 0x5d8, false),
        (2, 0x5e4, false),
        (3, 0x5e8, false),
        (4, 0x5f0, false),
        (5, 0x5f4, false),
    ],
    &[
        (0, 0x5fc, false),
        (1, 0x600, false),
        (2, 0x604, false),
        (3, 0x60c, false),
        (4, 0x620, false),
        (5, 0x61c, false),
        (6, 0x630, false),
        (7, 0x634, false),
        (8, 0x610, false),
    ],
    &[
        (1, 0x63c, false),
        (2, 0x650, false),
        (3, 0x654, false),
        (4, 0x658, false),
    ],
    &[
        (0, 0x65c, false),
        (1, 0x660, false),
        (2, 0x664, false),
        (3, 0x668, false),
        (4, 0x674, false),
        (5, 0x678, false),
    ],
    &[
        (0, 0x67c, false),
        (1, 0x684, false),
        (2, 0x688, false),
        (3, 0x68c, false),
    ],
    &[
        (0, 0x69c, false),
        (1, 0x6a0, false),
        (2, 0x6a4, false),
        (3, 0x6a8, false),
        (4, 0x6ac, false),
        (5, 0x6c8, false),
        (6, 0x6b8, false),
        (7, 0x6cc, false),
    ],
    &[
        (0, 0x6d0, false),
        (1, 0x6d4, false),
        (2, 0x6d8, false),
        (3, 0x6e0, false),
        (4, 0x6e4, false),
        (5, 0x6f0, false),
        (6, 0x6ec, false),
        (7, 0x700, false),
        (8, 0x704, false),
    ],
    &[
        (1, 0x70c, false),
        (2, 0x708, false),
        (3, 0x710, false),
        (4, 0x71c, false),
        (5, 0x73c, false),
        (6, 0x748, false),
        (7, 0x738, false),
        (8, 0x728, false),
    ],
    &[
        (0, 0x74c, false),
        (1, 0x754, false),
        (2, 0x768, false),
        (3, 0x76c, false),
    ],
    &[(1, 0x780, false), (4, 0x790, false)],
    &[
        (0, 0x794, true),
        (1, 0x798, true),
        (2, 0x79c, true),
        (3, 0x7a0, false),
        (4, 0x7a4, false),
        (5, 0x7a8, false),
        (6, 0x7ac, false),
    ],
    &[
        (1, 0x7b0, false),
        (2, 0x7e0, false),
        (3, 0x7b4, false),
        (4, 0x7e4, false),
        (5, 0x7b8, false),
    ],
    &[
        (0, 0x7e8, false),
        (1, 0x7ec, false),
        (2, 0x7f0, false),
        (3, 0x7f4, false),
        (4, 0x7fc, false),
        (5, 0x800, false),
        (7, 0x804, false),
    ],
    &[(0, 0x808, false), (1, 0x830, false)],
    &[(0, 0x848, false)],
    &[
        (0, 0x868, false),
        (1, 0x864, false),
        (2, 0x860, false),
        (3, 0x86c, false),
        (4, 0x870, false),
        (5, 0x874, false),
        (6, 0x878, false),
        (7, 0x880, false),
        (8, 0x884, false),
    ],
    &[(0, 0x88c, false), (1, 0x890, false), (2, 0x894, false)],
    &[
        (0, 0x8a0, false),
        (1, 0x8a8, false),
        (2, 0x8b0, false),
        (3, 0x8b4, false),
        (4, 0x8b8, false),
        (5, 0x8c0, false),
        (6, 0x8bc, false),
        (7, 0x8ac, false),
    ],
    &[(0, 0x8c8, false), (1, 0x8cc, false)],
];

#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tribe_parse.py") else {
        return;
    };
    let mut n = 0;
    let mut c1_base: Option<i64> = None;
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let tribe = rd(m, 0x54) as u32;
            let (b, per) = (rd(&c[0], 0x760), rd(&c[0], 0x764));
            let (mut s0, mut s1, mut s2, mut s3) = (
                String::from("-"),
                String::from("-"),
                String::from("-"),
                Vec::new(),
            );
            if let Some(rows) = TABLE.get(tribe as usize) {
                for &(i, off, abs) in rows.iter() {
                    let v = rd(&c[0], off);
                    wr(&mut c[2], 4 * i, if abs { v.wrapping_abs() } else { v });
                }
                if tribe == 13 {
                    // the raw address of the loc-string cell + offset: the base is the window's, one per run
                    let (off, n_arg, pe) = match (b != 0, per != 0) {
                        (true, true) => (0x118a0_i64, None, 0),
                        (false, true) => (0x118c8, Some(per), 2),
                        _ => (0x118f0, Some(b), 2),
                    };
                    let raw = {
                        let r = s1_raw(&want_log);
                        let base = r - off;
                        match c1_base {
                            None => c1_base = Some(base),
                            Some(x) => assert_eq!(x, base, "loc-string base — row `{line}`"),
                        }
                        r
                    };
                    let ans = |k: usize| i64::from(a[k]);
                    let _ = pe;
                    s1 = format!(
                        "sp-0x28,{raw}/sp-0x28,{}/15561040,sp-0x28",
                        if n_arg.is_some() { ans(2) } else { ans(0) }
                    );
                    if let Some(x) = n_arg {
                        s2 = format!("sp-0x28,sp-0x3c,{x}");
                    } else {
                        s0 = format!("sp-0x28,sp-0x3c,{b},{per}");
                    }
                    s3.push("sp-0x3c".to_string());
                }
            }
            s3.push("sp-0x28".to_string());
            log.extend([s0, s1, s2, s3.join("/")]);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "parse — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "parse — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "parse — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "parse — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Tribe::parse@006f0560: {n} rows agree");
}

/// The first operator= call's raw cell address on a Russian row (its second argument).
fn s1_raw(want_log: &[&str]) -> i64 {
    let first = want_log[1].split('/').next().expect("a call");
    first
        .split(',')
        .nth(1)
        .expect("an argument")
        .parse()
        .expect("an address")
}
