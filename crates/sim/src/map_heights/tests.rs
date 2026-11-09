use super::*;
use crate::mapgen::MapGrid;

/// `LineState`'s walk: both ends, one tile a step along the longer axis,
/// a point for a zero-length line.
#[test]
fn a_line_walks_its_longer_axis_end_to_end() {
    assert_eq!(line(3, 3, 3, 3), vec![(3, 3)]);
    assert_eq!(line(0, 0, 3, 0), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
    assert_eq!(line(0, 0, 0, -2), vec![(0, 0), (0, -1), (0, -2)]);
    assert_eq!(
        line(0, 0, 4, 2),
        vec![(0, 0), (1, 1), (2, 1), (3, 2), (4, 2)]
    );
    assert_eq!(line(0, 0, 1, 3), vec![(0, 0), (0, 1), (1, 2), (1, 3)]);
    assert_eq!(line(0, 0, 2, 2), vec![(0, 0), (1, 1), (2, 2)]);
}

/// `create_doobers` flattens an oil cell that carries the rock bit,
/// box and border as `terraform_for_building` draws them, and leaves
/// an oil cell without it, and every vertex outside the box, alone.
#[test]
fn an_oil_cell_with_rock_is_flattened_and_nothing_else_moves() {
    let mut g = MapGrid::new(8, 8);
    g.cells[3 * 8 + 3].flags |= OIL | ROCK;
    g.cells[6 * 8 + 1].flags |= OIL;
    let w = VERTS * g.xs + 1;
    let before: Vec<Single> = (0..w * w)
        .map(|i| Single::from_i32(10 + (i % w) * 3 + (i / w) * 7))
        .collect();
    let mut z = before.clone();
    create_doobers(&g, &mut z);
    let (x0, x1) = (11, 18);
    let inner = z[(14 * w + 14) as usize];
    for y in 0..w {
        for x in 0..w {
            let (b, a) = (before[(y * w + x) as usize], z[(y * w + x) as usize]);
            let inside = (x0..x1).contains(&x) && (x0..x1).contains(&y);
            if !inside {
                assert_eq!(a, b, "({x}, {y}) outside the box moved");
            } else if x == x0 || y == x0 || x == x1 - 1 || y == x1 - 1 {
                assert_eq!(a, b.addss(inner).mulss(Single::HALF), "({x}, {y}) border");
            } else {
                assert_eq!(a, inner, "({x}, {y}) interior");
            }
        }
    }
    assert_ne!(inner, before[(14 * w + 14) as usize]);
}
