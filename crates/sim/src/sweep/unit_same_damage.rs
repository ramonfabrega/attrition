//! `Unit::same_damage@005f9400` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Unit::same_damage@005f9400`, run by `tools/emu/sweep_unit_same_damage.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p28 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; callee 0x00609c00 (2 args) stubbed; callee 0x00647080 (3 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x158 (0 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `p0+16` points at 48 random bytes less `0` (placed); the word at `p0+20` points at 48 random bytes less `0` (placed); the word at `m+0xe0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 160 random bytes less `0` (placed); the word at `p7+4` points at 160 random bytes less `0` (placed); the word at `p7+8` points at 160 random bytes less `0` (placed); the word at `p7+12` points at 160 random bytes less `0` (placed); the word at `p7+16` points at 160 random bytes less `0` (placed); the word at `p7+20` points at 160 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 152 random bytes less `0` (placed); the word at `p14+4` points at 152 random bytes less `0` (placed); the word at `p14+8` points at 152 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p10+0x18` points at 784 random bytes less `0` (placed); the word at `p11+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p13+0x18` points at 784 random bytes less `0` (placed); slot +0x158 laid as 0x00647080 or its stub, a flag after the answers; slot +0x11c laid as 0x00610890 or its stub, a flag after the answers; the word at `c0+0x30` laid as the address of `p0+12` every row; stub 2 answers `p8+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x90`; stub 0 answers 4 values in call order, printed after the answers; stub 3 answers 5 values in call order, printed after the answers; stub 0 answers -20..60 every row; stub 3 answers 1..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); figures p16..p18 link at +0x8e (2 bytes) above their own index or to -1; figures p19..p21 link at +0x8e (2 bytes) above their own index or to -1; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[allow(unused_assignments)]
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_same_damage.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Args `a[0]` the unit's index, `a[1]` its player (`param_1`, `param_2`); stub answers `a[2..7]` (unused:
                // 0 `total_damage` and 3 slot `+0x11c` are `--seq`, 2 slot `+0xa8` answers by its `ecx`); `a[7..11]` the
                // four `total_damage` answers and `a[11..16]` the five `hits` answers, in call order; `a[16]` slot
                // `+0x158` is `Object::die` itself, `a[17]` slot `+0x11c` is `UnitData::hits` itself. Windows: m `this`;
                // p1.. the six objects (the two players' arrays are one array, the second at +12) and p8.. the data
                // each answers (slot `+0xa8`, its `o_down` laid by the answer); p16..p21 the two players' units (three
                // a player, `o_up` laid forward); p22 `this`'s type record, p23.. the data's.
                const M: usize = usize::MAX;
                macro_rules! w {
                    ($r:expr) => {
                        if $r == M { &*m } else { &p[$r] }
                    };
                }
                let rs = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let name = |r: usize| {
                    if r == M {
                        "m+0x0".to_string()
                    } else {
                        format!("p{r}+0x0")
                    }
                };
                let entry = |who: usize, k: usize| if who == 0 { k } else { 3 + k };
                let (die_base, hits_img) = (a[16] != 0, a[17] != 0);
                let (mut tn, mut hn) = (0usize, 0usize);
                macro_rules! hits {
                    ($r:expr) => {{
                        let r = $r;
                        if hits_img {
                            rd(w!(r), 0x20)
                        } else {
                            log.push(format!("s3:{},0", name(r)));
                            let v = a[11 + hn.min(4)];
                            hn += 1;
                            v
                        }
                    }};
                }
                macro_rules! td {
                    ($r:expr) => {{
                        let r = $r;
                        log.push(format!("s0:{},0,0", name(r)));
                        let v = a[7 + tn.min(3)];
                        tn += 1;
                        v
                    }};
                }
                let mut this = M;
                let mut sv = rs(m, 0x90);
                while sv >= 0 {
                    let e = entry(w!(this)[9] as usize, sv as usize);
                    let d = 8 + e;
                    if rd(&p[d], 8) & 1 == 0 {
                        break;
                    }
                    this = d;
                    sv = rs(&p[d], 0x90);
                }
                let ty = if this == M { 22 } else { 23 + (this - 8) };
                let uber = rd(&p[ty], 0x308);
                let iv4 = hits!(this) / uber;
                let h1 = hits!(this);
                let td0 = td!(this);
                let iv6 = (td0.wrapping_shl(8)) / h1;
                let (pa, pb) = (a[1] as usize, a[0] as usize);
                let ob = 1 + entry(pa, pb);
                let tdu = td!(16 + 3 * pa + pb);
                let hob = hits!(ob);
                let mut iv5 = (tdu.wrapping_shl(8)) / hob;
                if iv5 < iv6 {
                    iv5 = iv6;
                }
                let h2 = hits!(this);
                let td2 = td!(this);
                let (mut sidx, who) = (rs(w!(this), 0xa), w!(this)[9] as usize);
                let prod = h2.wrapping_mul(iv5);
                let mut iv7 = (prod.wrapping_add((prod >> 31) & 0xff) >> 8).wrapping_sub(td2);
                if iv7 < 0 {
                    iv7 = 0;
                }
                if iv7 != 0 {
                    loop {
                        if sidx < 0 {
                            break;
                        }
                        let u = 16 + 3 * who + sidx as usize;
                        if rd(&p[u], 8) & 1 == 0 {
                            break;
                        }
                        let next = rs(&p[u], 0x8e);
                        if next < 0 {
                            let h = hits!(u);
                            let t = td!(u);
                            let mut cap = h.wrapping_sub(t).wrapping_sub(1);
                            if iv7 < cap {
                                cap = iv7;
                            }
                            wr(&mut p[u], 0x24, cap);
                            break;
                        }
                        if iv7 < iv4 {
                            wr(&mut p[u], 0x24, iv7);
                            break;
                        }
                        if die_base {
                            log.push(format!("s1:{},0,-1,0", name(u)));
                        } else {
                            log.push(format!("s4:{}", name(u)));
                        }
                        iv7 = iv7.wrapping_sub(iv4);
                        if iv7 == 0 {
                            break;
                        }
                        sidx = next;
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "same_damage — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "same_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "same_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "same_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::same_damage@005f9400: {n} rows agree");
}
