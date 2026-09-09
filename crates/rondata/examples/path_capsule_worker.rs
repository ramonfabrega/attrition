//! Persistent semantic oracle for path reduction; one request and response per line.
use sim::{Pos, Unit, orders::PathData};
use std::io::{BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let words = line
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        let Some(&count) = words.first() else {
            return Err("missing count".into());
        };
        if count > 64 || words.len() != 1 + count as usize * 4 {
            return Err("invalid waypoint count".into());
        }
        let mut unit = Unit::new(0, 0, Pos { x: 0, y: 0 }, 1);
        for row in words[1..].chunks_exact(4) {
            if row[3] > 255 {
                return Err("invalid flags".into());
            }
            unit.path.push(PathData {
                to: Pos {
                    x: row[0] as i32,
                    y: row[1] as i32,
                },
                tolerance: row[2] as i32,
                flags: row[3] as u8,
            });
        }
        unit.discard_current_path_segment();
        write!(output, "{}", unit.path.len())?;
        for p in unit.path {
            write!(
                output,
                " {} {} {} {}",
                p.to.x as u32, p.to.y as u32, p.tolerance as u32, p.flags
            )?;
        }
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
