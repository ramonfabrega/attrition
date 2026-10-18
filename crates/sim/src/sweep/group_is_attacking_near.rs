//! `Group::is_attacking_near@00710e40` under the emulator against the port (item 3229; tools/sweep_gen.py, obj with windows).

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

/// `Group::is_attacking_near@00710e40`, run by `tools/emu/sweep_group_is_attacking_near.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 3 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; callee 0x00711540 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `c1+0` points at 256 random bytes less `0` (placed); the word at `c1+4` points at 256 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); the word at `p4+0x18` points at 704 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 1 answered from [1, 1, 1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 1, 1, 0] (90%); stub 4 answered from [0, 0, 0, 0, 0, 1] (90%); stub 5 answered from [0, 0, 0, 0, 0, 1] (90%); stub 6 answered from [0, 1, 2, 3, 4, 12, 5, 6, 7, 8, 9, 10, 11, 13, 18, 19, 21, 12, 0] (90%); stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 4 answers 6 values in call order, printed after the answers; stub 5 answers 6 values in call order, printed after the answers; stub 6 answers 6 values in call order, printed after the answers; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_is_attacking_near.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // m the group (`+0xc` count, `+0x4a` player, `+0x8cc` the shorts of its members); p1/p2 the
            // objects (`+0x10`/`+0x14` coordinates xor 0x63637), p3/p4 their unit records (`+0x8e` captain bit,
            // `+0x18` -> p5/p6 the type record, `+0x1e8`, `+0x2b8` flags). a[0], a[1] the point, a[2] unused;
            // a[10..16] slot +0x8, a[16..22] +0xbc, a[22..28] +0xe8, a[28..34] +0xcc, a[34..40] +0xc4,
            // a[40..46] `order_type`, each in call order, the last repeated; a[46..49] flags: the slot laid as
            // the base (`is_captain`, `is_supply`, `is_hero`) and read inline.
            let _ = c;
            let ld = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let vector_dist = |x: i32, y: i32| -> i32 {
                let ab = |v: i32| (v ^ (v >> 31)).wrapping_sub(v >> 31);
                let (e, s) = (ab(x), ab(y));
                let (eu, su) = (e as u32, s as u32);
                if e > s {
                    if e == 0 {
                        return 0;
                    }
                    if su >= 0xea60 {
                        return (su.wrapping_add(eu.wrapping_mul(2)) >> 1) as i32;
                    }
                    return (su.wrapping_mul(su) / eu.wrapping_mul(2)).wrapping_add(eu) as i32;
                }
                if s == 0 {
                    return 0;
                }
                if eu >= 0xea60 {
                    return (eu.wrapping_add(su.wrapping_mul(2)) >> 1) as i32;
                }
                (eu.wrapping_mul(eu) / su.wrapping_mul(2)).wrapping_add(su) as i32
            };
            let is_move = |t: i32| matches!(t, 1 | 2 | 3 | 4 | 0x12 | 0x13 | 0x15);
            assert_eq!(m[0x4a], 0, "the player is held to the cells' first row");
            let count = ld(m, 0xc);
            let (mut near, mut att) = (0i32, 0i32);
            let mut n = [0usize; 6];
            let seq = |k: usize, base: usize, n: &mut [usize; 6]| -> i32 {
                let v = a[base + n[k].min(5)];
                n[k] += 1;
                v
            };
            if count > 0 {
                for i in 0..count as usize {
                    let k = i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cd + 2 * i]]) as usize;
                    let (o, d, r) = (&p[1 + k], &p[3 + k], &p[5 + k]);
                    if seq(0, 10, &mut n) == 0 {
                        continue;
                    }
                    if seq(1, 16, &mut n) == 0 {
                        continue;
                    }
                    let captain = if a[46] != 0 {
                        i32::from(u16::from_le_bytes([d[0x8e], d[0x8f]]) >> 15)
                    } else {
                        seq(2, 22, &mut n)
                    };
                    if captain == 0 {
                        continue;
                    }
                    if ld(r, 0x1e8) == 0 {
                        continue;
                    }
                    let supply = if a[47] != 0 {
                        ld(r, 0x2b8) & 0x40
                    } else {
                        seq(3, 28, &mut n)
                    };
                    if supply != 0 {
                        continue;
                    }
                    let hero = if a[48] != 0 {
                        ld(r, 0x2b8) & 0x20
                    } else {
                        seq(4, 34, &mut n)
                    };
                    if hero != 0 {
                        continue;
                    }
                    let dy = a[1].wrapping_sub(ld(o, 0x14) ^ 0x63637);
                    let dx = a[0].wrapping_sub(ld(o, 0x10) ^ 0x63637);
                    let ab = |v: i32| (v ^ (v >> 31)).wrapping_sub(v >> 31);
                    if vector_dist(ab(dx), ab(dy)) > 0x600 {
                        continue;
                    }
                    near += 1;
                    let t = seq(5, 40, &mut n);
                    if !is_move(t) && t != 0 && t != 0xc {
                        att += 1;
                    }
                }
                i64::from(near != 0 && att >= near / 2)
            } else {
                0
            }
        };
        assert_eq!(got, want_eax, "is_attacking_near — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_attacking_near — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_attacking_near — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "is_attacking_near — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Group::is_attacking_near@00710e40: {n} rows agree");
}
