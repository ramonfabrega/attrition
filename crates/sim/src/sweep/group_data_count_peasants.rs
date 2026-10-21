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

/// `GroupData::count@00711720`, run by `tools/emu/sweep_group_data_count_peasants.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 3 argument(s), `p` windows in order: p0..p41 the place(s), in flag order; the global cell 0x00c0ab10 points at 2656 random bytes; callee 0x0046fa10 (0 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x0046d330 (0 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x0042db90 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 72 random bytes less `0` (placed); the word at `p5+4` points at 72 random bytes less `0` (placed); the word at `p5+8` points at 72 random bytes less `0` (placed); the word at `p5+12` points at 72 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 152 random bytes less `0` (placed); the word at `p10+4` points at 152 random bytes less `0` (placed); the word at `p10+8` points at 152 random bytes less `0` (placed); the word at `p10+12` points at 152 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `p15+12` points at 152 random bytes less `0` (placed); the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 16 random bytes less `0` (placed); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 16 random bytes less `0` (placed); the word at `p20+12` points at 16 random bytes less `0` (placed); the word at `c0+0xa4c` points at 16 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `p2+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); the word at `p4+0x18` points at 704 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); the word at `p9+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `p14+0x18` points at 704 random bytes less `0` (placed); the word at `p16+0x18` points at 704 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 5 answered from [6, 7, 7, 6, 0] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 440 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_count_peasants.py") else {
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
                // `GroupData::count`, arms COUNT_WORKERS (0), COUNT_PEASANTS (1), COUNT_GATHER_PEASANTS..
                // COUNT_CAN_GATHER (11..=15) and COUNT_CAN_BE_SCHOLARS (31), inlined: the crate has no `count` of
                // a group's list. Windows as the units slice: objects[who][i] is p[5w+1+i], units[who][i]
                // p[11+5w+i], their `+0x18` type records p[26+4w+i] / p[34+4w+i], the type table's records
                // p[21..25]; the object's companion (`+0x28` short, `+0x3e` signed byte: a table) is
                // objects[+0x3e][+0x28]. Stubs, each recorded by the `ecx` and arguments it is asked with and
                // answered in call order (a[13 + 8 s + j]): 0 is_worker, 1 is_peasant, 2 is_scholar,
                // 3 is_map_unit, 4 object slot 0x18, 5 action_type (GATHER 7, BUILD_AT 6), 6 is_castable
                // (the list value, the group's player, 0), 7 `TypeData::is_scholar`, 8 object slot 0xb8, 9 the
                // type record's slot 0x60 (`ObjectData::is` inline when the `--slotbase` flag a[93] is set).
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let mut turn = [0usize; 10];
                let mut calls: Vec<Vec<String>> = vec![Vec::new(); 10];
                let mut ask = |s: usize, ecx: &str, args: &str| -> i32 {
                    calls[s].push(format!("{ecx}{args}"));
                    let v = a[13 + 8 * s + turn[s].min(7)];
                    turn[s] += 1;
                    v
                };
                let mut cnt = 0i32;
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                            .expect("an index");
                    let obj = &p[5 * who + 1 + i];
                    let unit = &p[11 + 5 * who + i];
                    let ty = &p[34 + 4 * who + i];
                    if rd(obj, 8) & 1 == 0 {
                        continue;
                    }
                    let on = format!("p{}+0x0", 5 * who + 1 + i);
                    let un = format!("p{}+0x0", 11 + 5 * who + i);
                    let tn = format!("p{}+0x0", 26 + 4 * who + i);
                    // The companion object, when `+0x28` is not negative.
                    let comp = {
                        let nx = i16::from_le_bytes([obj[0x28], obj[0x29]]);
                        (nx >= 0).then(|| {
                            let nw = usize::try_from(obj[0x3e] as i8).expect("a table");
                            format!(
                                "p{}+0x0",
                                5 * nw + 1 + usize::try_from(nx).expect("an index")
                            )
                        })
                    };
                    match a[0] {
                        0 => {
                            cnt += i32::from(ask(0, &on, "") != 0);
                            if a[1] != 0
                                && ask(3, &on, "") != 0
                                && (rd(ty, 0x2b4) & 0x10) != 0
                                && let Some(c) = &comp
                            {
                                cnt += i32::from(ask(0, c, "") != 0);
                            }
                        }
                        1 => {
                            cnt += i32::from(ask(1, &on, "") != 0);
                            if ask(3, &on, "") != 0
                                && let Some(c) = &comp
                            {
                                cnt += i32::from(ask(1, c, "") != 0);
                            }
                        }
                        11..=13 => {
                            if ask(4, &on, "") != 0 {
                                let r = match a[0] {
                                    11 => ask(1, &un, ""),
                                    12 => ask(0, &un, ""),
                                    _ => ask(2, &un, ""),
                                };
                                if r != 0 && ask(5, &un, "") == 7 {
                                    cnt += 1;
                                }
                            }
                        }
                        14 => {
                            if ask(1, &on, "") != 0 && ask(5, &un, "") == 6 {
                                cnt += 1;
                            }
                        }
                        15 => {
                            let args = format!(",{i},{who},0");
                            let mut ok = ask(1, &on, "") != 0;
                            if !ok {
                                ok = ask(6, "p25+0x0", &args) != 0;
                            }
                            cnt += i32::from(ok);
                            if ask(3, &on, "") != 0
                                && let Some(c) = &comp
                                && (ask(1, c, "") != 0 || ask(6, "p25+0x0", &args) != 0)
                            {
                                cnt += 1;
                            }
                        }
                        31 => {
                            cnt += i32::from(ask(2, &on, "") != 0);
                            let is = if a[93] != 0 {
                                ask(9, &tn, ",66,0")
                            } else {
                                ask(8, &on, ",66,0")
                            };
                            if is != 0 {
                                let rec = format!("p{}+0x0", 21 + rd(unit, 0x54));
                                if ask(7, &rec, "") != 0 {
                                    cnt += 1;
                                }
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
            assert_eq!(w, &want[2 + i], "count — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 440, "only {n} rows");
    eprintln!("GroupData::count@00711720: {n} rows agree");
}
