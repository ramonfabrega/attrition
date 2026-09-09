//! Decode original-issued packets from the local command_oracle.py experiment.
use rondata::commands::{Command, decode};
use std::io::{self, BufRead};

fn main() {
    let mut count = 0;
    for line in io::stdin().lock().lines() {
        let line = line.expect("read oracle row");
        let row: Vec<_> = line.split('\t').collect();
        assert_eq!(row.len(), 12, "malformed row");
        let label = row[0];
        let n: Vec<i32> = row[2..11].iter().map(|s| s.parse().unwrap()).collect();
        let packet = if row[11] == "-" {
            Vec::new()
        } else {
            assert_eq!(row[11].len() % 2, 0);
            (0..row[11].len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&row[11][i..i + 2], 16).unwrap())
                .collect()
        };
        let mut want = Vec::new();
        match row[1] {
            "none" => {}
            "new" | "group" => want.push(Command::Group {
                who: 0,
                objects: vec![1],
            }),
            "reuse" => want.push(Command::Group {
                who: 0,
                objects: vec![],
            }),
            other => panic!("unknown expectation {other}"),
        }
        if matches!(row[1], "new" | "reuse") {
            want.push(Command::MoveTo {
                to_x: n[0],
                to_y: n[1],
                queued: n[2].try_into().unwrap(),
                set_angle: n[3],
                angle: n[4],
                orders: n[5].try_into().unwrap(),
                form: n[6].try_into().unwrap(),
                width: n[7].try_into().unwrap(),
                disembark: n[8].try_into().unwrap(),
            });
        }
        assert_eq!(decode(&packet).expect(label), want, "{label}");
        count += 1;
    }
    assert!(count >= 35, "missing oracle rows: {count}");
    println!("{count} original command packets matched Rust decoding");
}
