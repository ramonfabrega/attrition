//! Compare the local executable's generated turn-angle table against the Rust port.
use sim::movement::{Angle, TurnMode, Turning, turn_speed, turn_towards};
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let row = line
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(row.len(), 13);
        let turning = Turning {
            type_turn_speed: row[0] as i32,
            packed: row[1] != 0,
            instant_from_stop: row[2] != 0,
            ..Turning::default()
        };
        let mut tuning = sim::Tuning::RON;
        tuning.unit_turn_speed = row[6] as i32;
        tuning.unit_pack_turn_bonus = row[7] as i32;
        let mode = if row[5] == 0 {
            TurnMode::Unit
        } else {
            TurnMode::Body
        };
        let mut rate = turn_speed(&tuning, &turning, row[3] as i32, row[4] as i32, mode);
        if row[10] != 0 {
            rate >>= 1;
        }
        let (heading, remaining) = turn_towards(Angle(row[8] as i32), Angle(row[9] as i32), rate);
        assert_eq!(
            (heading.0 as u32, remaining),
            (row[11], row[12]),
            "row {count}: {line}"
        );
        count += 1;
    }
    assert!(count > 0, "empty oracle table");
    println!("{count} original/Rust turn-angle output/return comparisons passed");
    Ok(())
}
