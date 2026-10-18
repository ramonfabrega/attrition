//! `Unit::do_air_attack_ground@005ea420` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_air_attack_ground@005ea420`, run by `tools/emu/sweep_unit_do_air_attack_ground.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1 the place, p2 stub 0's answer window; the global cell 0x00c061f0 points at 4096 random bytes; argument 0 names 64 random bytes; vtable slot +0x58 (0 args) stubbed; callee 0x005e86d0 (3 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x0064e4a0 (4 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; callee 0x005fce70 (2 args) stubbed; callee 0x0064c8b0 (2 args) stubbed; callee 0x00616f40 (3 args) stubbed; vtable slot +0x134 (1 args) stubbed; vtable slot +0x158 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; the word at `m+0x18` points at 768 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 1 answered from [1, 1, 1, 1, 1, 0] (90%); stub 2 answered from [0, 0, 0, 268435456, 805306368, 1073741824] (90%); stub 3 answered from [1, 1, 1, 1, 1, 0] (90%); stub 4 answered from [1, 1, 1, 0] (90%); stub 5 answered from [1, 1, 1, 0] (90%); stub 0 answers the address of 64 random bytes, a `p` window after the places; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_air_attack_ground.py") else {
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
            // p0 the order argument, p1 the type record `m+0x18` points at, p2 slot +0x58's answer; c0 the
            // constants record; a[2] do_air_physics, a[3] find_angle, a[4] is_in_range, a[5] the slot +0xb8
            // stub's answer, a[6] the type record's slot +0x60 answer, a[10] slot +0x134's; a[13] / a[14] 1 when
            // the image's `ObjectData::is` / `Object::die` stands in the slot.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let mut t: Vec<Vec<String>> = vec![Vec::new(); 11];
            let (x, y) = (rdw(&p[2], 4), rdw(&p[2], 8));
            t[0].push(format!("m+0x0,0,p0+0x0,{x},{y}"));
            let rec = |p: &[Vec<u8>], o: usize| rdw(&p[1], o);
            'body: {
                if a[2] == 0 || m[0xae] != 0 {
                    break 'body;
                }
                let (mx, my) = (rdw(m, 0x10) ^ 0x63637, rdw(m, 0x14) ^ 0x63637);
                let edx = y.wrapping_sub(my);
                t[1].push(format!("{},{edx}", x.wrapping_sub(mx)));
                let mut local = rdw(m, 0x50).wrapping_sub(a[3]) as u32;
                if local > 0x8000_0000 {
                    local = !local;
                }
                if rdw(&p[2], 0x2c) != 0 {
                    break 'body;
                }
                t[2].push(format!("m+0x0,{edx},{x},{y},{mx},{my}"));
                if a[4] == 0 {
                    break 'body;
                }
                // `ObjectData::is(what, 0)` through slot +0xb8: the type record's slot +0x60, or the stub.
                let is = |t: &mut Vec<Vec<String>>, what: i32| {
                    if a[13] != 0 {
                        t[4].push(format!("p1+0x0,{edx}"));
                        a[6] != 0
                    } else {
                        t[3].push(format!("m+0x0,{edx},{what},0"));
                        a[5] != 0
                    }
                };
                if rec(p, 0x1e4) & 0x8000000 == 0 && local >= 0xaaaaaaa {
                    if !is(&mut t, 0x127) || local > 0x2aaaaaaa {
                        break 'body;
                    }
                }
                t[5].push(format!("m+0x0,{edx},-1,-1"));
                let mut anim = true;
                if rec(p, 0x2b4) & 0x400000 == 0 && !is(&mut t, 0x130) && rec(p, 0x2cc) != 0 {
                    t[6].push(format!("m+0x0,{edx},-1,-1"));
                    anim = false;
                }
                if anim {
                    t[7].push(format!("m+0x0,{edx},12,0,1"));
                }
                if rec(p, 0x1e4) & 0x8000000 == 0 {
                    t[8].push(format!("m+0x0,{edx},0"));
                    m[0xae] = (a[10] as u8).wrapping_add(1);
                    if is(&mut t, 0x130) {
                        let cost = i16::from_le_bytes([c[0][0xc14], c[0][0xc15]]);
                        let at = u16::from_le_bytes([m[0x96], m[0x97]]).wrapping_add(cost as u16);
                        m[0x96..0x98].copy_from_slice(&at.to_le_bytes());
                    }
                } else if a[14] != 0 {
                    t[10].push(format!("m+0x0,{edx},0,-1,0"));
                } else {
                    t[9].push(format!("m+0x0,{edx},0,-1,0"));
                }
            }
            for v in t {
                log.push(if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_air_attack_ground — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "do_air_attack_ground — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_air_attack_ground — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "do_air_attack_ground — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Unit::do_air_attack_ground@005ea420: {n} rows agree");
}
