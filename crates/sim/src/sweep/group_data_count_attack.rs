//! `GroupData::count@00711720` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::count@00711720`, run by `tools/emu/sweep_group_data_count_attack.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 3 argument(s), `p` windows in order: p0..p35 the place(s), in flag order; vtable slot +0xe8 (0 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xd0 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 72 random bytes less `0` (placed); the word at `p5+4` points at 72 random bytes less `0` (placed); the word at `p5+8` points at 72 random bytes less `0` (placed); the word at `p5+12` points at 72 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 152 random bytes less `0` (placed); the word at `p10+4` points at 152 random bytes less `0` (placed); the word at `p10+8` points at 152 random bytes less `0` (placed); the word at `p10+12` points at 152 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `p15+12` points at 152 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `p2+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); the word at `p4+0x18` points at 704 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); the word at `p9+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `p14+0x18` points at 704 random bytes less `0` (placed); the word at `p16+0x18` points at 704 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 440 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_count_attack.py") else {
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
                // `GroupData::count`, arms COUNT_ATTACK (2), COUNT_MILITARY (3) and COUNT_TRADE (22), inlined: the
                // crate has no `count` of a group's list. Windows as the units slice (objects p[5w+1+i], units
                // p[11+5w+i], object types p[20+4w+i], unit types p[28+4w+i]); an object's companion
                // (`+0x28` short, `+0x3e` signed byte: a table) is objects[+0x3e][+0x28]. Stubs, each recorded and
                // answered in call order (a[10 + 8 s + j]): 0 the object's slot 0xe8, 1 `ObjectData::is_map_unit`,
                // 2 slot 0xb8, 3 the type record's slot 0x60 (`ObjectData::is` inline when the `--slotbase` flag
                // a[66] is set), 4 slot 0x8, 5 slot 0x148 (handed 4), 6 slot 0xd0.
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let mut turn = [0usize; 7];
                let mut calls: Vec<Vec<String>> = vec![Vec::new(); 7];
                let mut ask = |s: usize, ecx: &str, args: &str| -> i32 {
                    calls[s].push(format!("{ecx}{args}"));
                    let v = a[10 + 8 * s + turn[s].min(7)];
                    turn[s] += 1;
                    v
                };
                // `is(ty, 0)` of the record named `rec` whose type record is `ty_rec`.
                let is = |ask: &mut dyn FnMut(usize, &str, &str) -> i32,
                          rec: &str,
                          ty_rec: &str,
                          ty: i32| {
                    let args = format!(",{ty},0");
                    if a[66] != 0 {
                        ask(3, ty_rec, &args)
                    } else {
                        ask(2, rec, &args)
                    }
                };
                let mut cnt = 0i32;
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                            .expect("an index");
                    let obj = &p[5 * who + 1 + i];
                    let oty = &p[20 + 4 * who + i];
                    let uty = &p[28 + 4 * who + i];
                    if rd(obj, 8) & 1 == 0 {
                        continue;
                    }
                    let on = format!("p{}+0x0", 5 * who + 1 + i);
                    let un = format!("p{}+0x0", 11 + 5 * who + i);
                    let otn = format!("p{}+0x0", 20 + 4 * who + i);
                    let utn = format!("p{}+0x0", 28 + 4 * who + i);
                    match a[0] {
                        2 => {
                            if rd(oty, 0x1e8) == 0 || ask(0, &on, "") == 0 {
                                if ask(1, &on, "") != 0 && is(&mut ask, &on, &otn, 0x140) != 0 {
                                    let nx = i16::from_le_bytes([obj[0x28], obj[0x29]]);
                                    if let Ok(nx) = usize::try_from(nx) {
                                        let nw = usize::try_from(obj[0x3e] as i8).expect("a table");
                                        let comp = &p[5 * nw + 1 + nx];
                                        let cty = &p[20 + 4 * nw + nx];
                                        if rd(comp, 8) & 1 != 0 && rd(cty, 0x1e8) != 0 {
                                            let cn = format!("p{}+0x0", 5 * nw + 1 + nx);
                                            if ask(0, &cn, "") != 0 {
                                                cnt += 1;
                                            }
                                        }
                                    }
                                }
                            } else {
                                cnt += 1;
                            }
                        }
                        3 => {
                            if ask(4, &on, "") != 0 {
                                if rd(uty, 0x1e8) == 0 || ask(5, &on, ",4") != 0 {
                                    if is(&mut ask, &un, &utn, 0x36) != 0
                                        || is(&mut ask, &un, &utn, 0x3f) != 0
                                    {
                                        cnt += 1;
                                    }
                                } else {
                                    cnt += 1;
                                }
                            }
                        }
                        22 => {
                            if ask(6, &on, "") != 0 || is(&mut ask, &on, &otn, 0x13e) != 0 {
                                cnt += 1;
                            }
                        }
                        _ => unreachable!("argset"),
                    }
                }
                for c in &calls {
                    log.push(if c.is_empty() {
                        "-".into()
                    } else {
                        c.join("/")
                    });
                }
                i64::from(cnt)
            }
        };
        assert_eq!(got, want_eax, "count — eax, row `{line}`");
        assert_eq!(log, want_log, "count — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 440, "only {n} rows");
    eprintln!("GroupData::count@00711720: {n} rows agree");
}
