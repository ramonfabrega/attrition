//! `ObjectData::is_jammed@00653660` under the emulator against the port (item 2714; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::is_jammed@00653660`, run by `tools/emu/sweep_object_data_is_jammed.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), argument 0 names 4 random bytes (null a third of the rows); vtable slot +0x148 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x00740070 (4 args) stubbed; the word at `m+0x18` points at 540 random bytes less `0` (placed); the word at `p9+0` points at 8 random bytes less `0` (placed); the word at `p10+0` points at 8 random bytes less `0` (placed); the word at `p11+0` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 8 random bytes less `0` (placed); the word at `p13+0` points at 8 random bytes less `0` (placed); the word at `p14+0` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 8 random bytes less `0` (placed); the word at `p16+0` points at 8 random bytes less `0` (placed); the word at `p18+0` points at 48 random bytes less `0` (placed); the word at `p18+4` points at 48 random bytes less `0` (placed); the word at `p19+0` points at 48 random bytes less `0` (placed); the word at `p19+4` points at 48 random bytes less `0` (placed); the word at `p20+0` points at 48 random bytes less `0` (placed); the word at `p20+4` points at 48 random bytes less `0` (placed); the word at `p21+0` points at 48 random bytes less `0` (placed); the word at `p21+4` points at 48 random bytes less `0` (placed); the word at `p22+0` points at 48 random bytes less `0` (placed); the word at `p22+4` points at 48 random bytes less `0` (placed); the word at `p23+0` points at 48 random bytes less `0` (placed); the word at `p23+4` points at 48 random bytes less `0` (placed); the word at `p24+0` points at 48 random bytes less `0` (placed); the word at `p24+4` points at 48 random bytes less `0` (placed); the word at `p25+0` points at 48 random bytes less `0` (placed); the word at `p25+4` points at 48 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 2 answers -1..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc0a290`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_jammed.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let me = ((rd(m, 8) >> 8) & 0xff) as usize;
                let fl = |k: usize| rd(&p[1 + k], 0);
                let mut tok = vec!["-".to_string(); 18];
                let mut r: i64 = -1;
                if fl(me) & 0x40000 != 0 && rd(&p[17], 0x218) != 2 && a[1] != 0 {
                    let (mut ally_i, mut fs_i) = (0usize, 0usize);
                    for k in 0..8usize {
                        if fl(k) & 1 == 0 || k == me {
                            continue;
                        }
                        tok[k] = format!("r+0x0,{me}");
                        let ally = a[4 + ally_i.min(7)];
                        ally_i += 1;
                        if ally != 0 || fl(k) & 0x20000 == 0 {
                            continue;
                        }
                        let x = rd(m, 0x10) ^ 0x63637;
                        let t = rd(m, 0x14) ^ 0x63637;
                        let call = format!("{t},{x},{t},{k},{t}");
                        if tok[17] == "-" {
                            tok[17] = call;
                        } else {
                            tok[17] = format!("{}/{call}", tok[17]);
                        }
                        let s = a[12 + fs_i.min(7)];
                        fs_i += 1;
                        if s >= 0 {
                            let rec = &p[26 + 2 * k + s as usize];
                            r = i64::from(i16::from_le_bytes([rec[0x26], rec[0x27]]));
                            if a[0] != 0 {
                                wr(&mut p[0], 0, k as i32);
                            }
                            break;
                        }
                    }
                }
                log.extend(tok);
                r
            }
        };
        assert_eq!(got, want_eax, "is_jammed — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_jammed — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_jammed — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_jammed — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_jammed — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::is_jammed@00653660: {n} rows agree");
}
