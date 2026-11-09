use super::*;

/// The knots of a clamped cubic on five points: four zeros, 1, 2,
/// then 2 held to the ninth.
#[test]
fn the_knots_are_clamped() {
    let k: Vec<i32> = Spline::knots(5, 3).iter().map(|s| s.to_i32()).collect();
    assert_eq!(k, vec![0, 0, 0, 0, 1, 2, 2, 2, 2]);
}

/// A straight spline stays on its line, starts and ends on its end
/// points, and its normals are the line's left-hand perpendicular.
#[test]
#[allow(clippy::float_arithmetic)]
fn a_straight_spline_stays_straight() {
    let mut s = Spline {
        control: (0..5)
            .map(|i| V3::new(centre(i), centre(2), Single::ZERO))
            .collect(),
        ..Spline::default()
    };
    s.calc(8);
    assert_eq!(s.verts.first(), s.control.first());
    assert_eq!(s.verts.last(), s.control.last());
    assert!(s.verts.iter().all(|v| v.y == centre(2)));
    assert!(s.verts.windows(2).all(|w| w[1].x.gt(w[0].x)));
    assert_eq!(s.normals.len(), s.verts.len());
    for n in &s.normals {
        assert_eq!((n.x.eq_value(Single::ZERO), n.y), (true, MINUS_ONE));
    }
    assert_eq!(s.total, Single::from_i32(4 * 768));
}

/// The walk visits the start, then steps of `step` until the end.
#[test]
fn next_at_dist_steps_to_the_end() {
    let mut s = Spline {
        control: (0..4)
            .map(|i| V3::new(centre(i), centre(0), Single::ZERO))
            .collect(),
        ..Spline::default()
    };
    s.calc(6);
    s.rewind();
    let (p, _, more) = s.next_at_dist(Single::ZERO, STEP_RATIO);
    assert_eq!((p, more), (s.verts[0], true));
    let mut n = 0;
    loop {
        let (_, _, more) = s.next_at_dist(Single::from_i32(500), STEP_RATIO);
        n += 1;
        if !more {
            break;
        }
        assert!(n < 100);
    }
    // 2,304 world units in steps of 500: four whole, the last short.
    assert_eq!(n, 5);
}
