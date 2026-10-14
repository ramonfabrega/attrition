//! `TypeData::get_limiting_resource@00667730` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

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

/// `TypeData::get_limiting_resource@00667730`, run by `tools/emu/sweep_type_data_get_limiting_resource.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3, 4, 5, 100] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_get_limiting_resource.py") else {
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
                // Over the six goods: one leader a0 has (type_avail(good, 1) on its
                // row) whose need (this's slot +0x78: good, a0, a1, a2, 0, 1, -1) is
                // more than its stock (the row's +0x6eb8 list, each word xored with
                // 0x8221) is the limiting resource; a second answers -1.
                let row = a[0] as usize;
                let (mut avail, mut need) = (5usize, 11usize);
                let mut lim = -1i64;
                let (mut ta, mut sl) = (Vec::new(), Vec::new());
                let mut v = None;
                for good in 0..6 {
                    ta.push(format!("r+0x0,{good},1"));
                    let ok = a[avail] != 0;
                    avail = (avail + 1).min(10);
                    if !ok {
                        continue;
                    }
                    sl.push(format!("m+0x0,{good},{},{},{},0,1,-1", a[0], a[1], a[2]));
                    let n = a[need];
                    need = (need + 1).min(16);
                    if (rd(&p[8 + row], 4 * good) ^ 0x8221) < n {
                        if lim != -1 {
                            v = Some(-1);
                            break;
                        }
                        lim = good as i64;
                    }
                }
                let mut rows = vec!["-".to_string(); 9];
                rows[row] = ta.join("/");
                log.extend(rows);
                log.push(if sl.is_empty() {
                    "-".into()
                } else {
                    sl.join("/")
                });
                v.unwrap_or(lim)
            }
        };
        assert_eq!(got, want_eax, "get_limiting_resource — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_limiting_resource — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_limiting_resource — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_limiting_resource — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_limiting_resource — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::get_limiting_resource@00667730: {n} rows agree");
}
