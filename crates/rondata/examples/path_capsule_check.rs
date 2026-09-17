//! Compare complete surviving waypoints with an original path-capsule oracle.
use sim::{Pos, Unit, orders::PathData};
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut checked = 0;
    let mut retained = 0;
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let fields = line
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        assert!(fields.len() >= 2, "missing row header");
        let count = fields[0] as usize;
        let remaining = fields[1] as usize;
        assert!(count <= 64 && remaining <= count, "outside capsule domain");
        assert_eq!(fields.len(), 2 + count * 4, "incomplete waypoint record");
        let mut unit = Unit::new(0, 0, Pos { x: 0, y: 0 }, 1);
        for row in fields[2..].chunks_exact(4) {
            assert!(row[3] <= u8::MAX as u32);
            unit.path.push(PathData {
                to: Pos {
                    x: row[0] as i32,
                    y: row[1] as i32,
                },
                tolerance: row[2] as i32,
                flags: row[3] as u8,
            });
        }
        let expected = unit.path[..remaining].to_vec();
        unit.discard_current_path_segment();
        assert_eq!(unit.path, expected, "row {checked}: {line}");
        checked += 1;
        retained += usize::from(remaining != 0);
    }
    assert!(checked > 0, "empty oracle table");
    println!("{checked} original/Rust path comparisons passed; {retained} retain waypoints");
    Ok(())
}
