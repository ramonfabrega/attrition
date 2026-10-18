//! `Unit::do_patrol@005f1910` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_patrol@005f1910`, run by `tools/emu/sweep_unit_do_patrol.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p7 the place(s), in flag order, p8 stub 2's answer window; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00e85f20 points at 12288 random bytes; argument 0 names 16 random bytes; vtable slot +0xa4 (0 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x44 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x00616f40 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x007111c0 (0 args) stubbed; stub 0's answer points at 96 random bytes less `0` (placed); the word at `p1+0x14` points at 64 random bytes less `0` (placed); the word at `p1+0x30` points at 64 random bytes less `0` (placed); stub 3's answer points at 96 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 9 answered from [0, 1] (90%); stub 10 answered from [0, 1] (90%); stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_patrol.py") else {
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
            {
                // Windows: m the unit, c0 `div_3_table`, c1 `groups`' records (stride 0x9d4); p0 the order argument,
                // p1 the patrol record (`+0x14` the x points p2, `+0x30` the y points p3), p4 the pushed `ATTACK_TO`
                // order (its vtable p5), p6 what `+0xdc` points at, p7 the `+0x18` object. Stub answers: `a[2]` the
                // angle, `a[10]`/`a[11]` slots `+0xb8` / `+0x60`, `a[15]` whether `+0xb8` held `ObjectData::is`.
                // One column a stub, its calls `ecx,edx,args..` -- `edx` is the stale register the original leaves.
                const K: i32 = 0x63637;
                let mut cols: Vec<Vec<String>> = vec![Vec::new(); 14];
                cols[0].push("p0+0x0,0".to_string());
                let mut edx = "p1+0x0".to_string();
                let grp = i16::from_le_bytes([m[0x80], m[0x81]]);
                let step = |p: &mut [Vec<u8>]| -> usize {
                    let mut i = rd(&p[1], 0x3c).wrapping_add(1);
                    if rd(&p[1], 8) <= i {
                        i = 0;
                    }
                    wr(&mut p[1], 0x3c, i);
                    i as usize
                };
                if grp < 0 {
                    let i = step(p);
                    let (xv, yv) = (rd(&p[2], 4 * i), rd(&p[3], 4 * i));
                    cols[1].push(format!(
                        "{},{}",
                        xv.wrapping_sub(rd(m, 0x10) ^ K),
                        yv.wrapping_sub(rd(m, 0x14) ^ K)
                    ));
                    let ang = a[2];
                    let xt = rd(&c[0], 4 * (xv >> 4) as usize);
                    let yt = rd(&c[0], 4 * (yv >> 4) as usize);
                    cols[2].push(format!("{yt},c0+0x0,2"));
                    // `edx` is the stubbed vtable's address, then the last `imul`'s high word >> 7.
                    cols[3].push("p8+0x0,536870912".to_string());
                    let x4 = xt.wrapping_mul(0x30).wrapping_add(0x18);
                    let y4 = yt.wrapping_mul(0x30).wrapping_add(0x18);
                    let o = &mut p[4];
                    wr(o, 4, x4);
                    wr(o, 8, y4);
                    wr(o, 0xc, ang);
                    wr(o, 0x10, 0);
                    wr(o, 0x2c, x4);
                    wr(o, 0x30, y4);
                    wr(o, 0x34, -1);
                    wr(o, 0x38, -1);
                    let off = rd(&p[5], 4) as usize;
                    let o = &mut p[4];
                    o[off + 4] &= 0xfe;
                    // The shorts reload `+0x4` / `+0x8` after the flag byte was cleared in them.
                    let (xs, ys) = (rd(o, 4), rd(o, 8));
                    o[0x4c..0x4e].copy_from_slice(&((xs % 0x300) as i16).to_le_bytes());
                    o[0x4e..0x50].copy_from_slice(&((ys % 0x300) as i16).to_le_bytes());
                    wr(o, 0x44, -1);
                    wr(o, 0x48, -1);
                    wr(o, 0x18, 0);
                    wr(o, 0x1c, 0);
                    wr(o, 0x24, 0);
                    wr(o, 0x28, -1);
                    o[off + 4] &= 0xfb;
                    o[off + 4] &= 0xdf;
                    let q = ((i64::from(ys) * 0x2aaaaaab) >> 32) >> 7;
                    edx = q.to_string();
                    cols[4].push(format!("m+0xcc,{edx},p4+{off:#x}"));
                    cols[5].push(format!("m+0x0,{edx}"));
                    let v = rd(&p[6], 0);
                    wr(m, 0xdc, v);
                    cols[6].push(format!("m+0x0,{edx}"));
                } else if rd(&p[1], 0x44) == i32::from(i16::from_le_bytes([m[0xa], m[0xb]]))
                    && rd(&p[1], 0x48) == i32::from(m[9])
                {
                    let i = step(p);
                    cols[7].push(format!(
                        "c1+{:#x},{edx},{},{},0,0,0,2,0,-1,-1,0",
                        i32::from(grp) * 0x9d4,
                        rd(&p[2], 4 * i),
                        rd(&p[3], 4 * i)
                    ));
                } else {
                    cols[8].push(format!("m+0x0,{edx},0,0,1"));
                }
                if i16::from_le_bytes([m[0x28], m[0x29]]) >= 0 {
                    let ans = if a[15] != 0 {
                        cols[10].push(format!("p7+0x0,{edx}"));
                        a[11]
                    } else {
                        cols[9].push(format!("m+0x0,{edx},351,1"));
                        a[10]
                    };
                    if ans != 0 {
                        let g = "sp-0x9ec";
                        cols[11].push(format!("{g},{edx},-1"));
                        let id = i16::from_le_bytes([m[0xa], m[0xb]]);
                        cols[12].push(format!("{g},{edx},{id},{},0,0", m[9]));
                        cols[13].push(format!("{g},{edx}"));
                    }
                }
                for col in &cols {
                    log.push(if col.is_empty() {
                        "-".to_string()
                    } else {
                        col.join("/")
                    });
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_patrol — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_patrol — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_patrol — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "do_patrol — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::do_patrol@005f1910: {n} rows agree");
}
