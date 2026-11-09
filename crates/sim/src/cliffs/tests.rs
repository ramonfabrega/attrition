use super::*;

/// [`SLOPE`] is the original's arithmetic in native single precision:
/// `25 · 0x3c8efa35`, `tanf`, `· 192` — the test oracle for the pin.
#[test]
fn the_slope_threshold_is_192_tanf_25_degrees() {
    let a = 25.0f32 * f32::from_bits(0x3c8e_fa35);
    let t = a.tan() * 192.0f32;
    assert_eq!(t.to_bits(), SLOPE.0, "{t}");
}

/// The span is strict and the first corner is clamped one way each: a
/// tile whose corners span exactly the threshold fails, one ulp more
/// passes; a flat tile at 20000 passes, its first corner's low clamp
/// reading 10000 against the others' 20000, and so does one at −20000.
#[test]
fn the_slope_test_is_strict_and_clamps_the_first_corner() {
    let w = 2;
    let flat = |a: Single, b: Single| vec![a, b, Single::ZERO, Single::ZERO];
    assert!(!has_plus_slope(&flat(SLOPE, Single::ZERO), w, 0, 0));
    assert!(has_plus_slope(
        &flat(Single(SLOPE.0 + 1), Single::ZERO),
        w,
        0,
        0
    ));
    assert!(has_plus_slope(
        &flat(Single::ZERO, Single(SLOPE.0 + 1)),
        w,
        0,
        0
    ));
    for n in [20000, -20000] {
        let v = Single::from_i32(n);
        assert!(has_plus_slope(&[v; 4], w, 0, 0), "a flat tile at {n}");
    }
    let v = Single::from_i32(9000);
    assert!(
        !has_plus_slope(&[v; 4], w, 0, 0),
        "a flat tile inside the clamp"
    );
}
