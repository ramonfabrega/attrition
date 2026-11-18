use super::*;
use rondata::commands::{Command, decode};

fn empty() -> Recording {
    Recording {
        path: PathBuf::new(),
        header: RecGame::default(),
        leader: crate::input::Input::default().leader_options(0),
        orders: vec![],
        paused: false,
    }
}

#[test]
fn turns_preserve_order_and_pause_boundaries() {
    let mut r = empty();
    let order = |cmd| (30, 0, vec![cmd]);
    r.accept(&order(Command::Stance { stance: 1 }), false)
        .unwrap();
    r.accept(&order(Command::Pause { state: 1 }), true).unwrap();
    r.accept(&order(Command::Pause { state: 0 }), false)
        .unwrap();
    r.accept(&order(Command::Stance { stance: 2 }), false)
        .unwrap();
    let packages = recgame::minted_packages(r.leader, &r.orders, 31).unwrap();
    let frames: Vec<_> = packages.iter().map(|p| p.frame).collect();
    assert_eq!(frames, [0, 30, 30, 31]);
    assert_eq!(
        decode(&packages[1].data).unwrap(),
        [Command::Stance { stance: 1 }, Command::Pause { state: 1 }]
    );
    assert_eq!(
        decode(&packages[2].data).unwrap(),
        [Command::Pause { state: 0 }, Command::Stance { stance: 2 }]
    );
}

#[test]
fn a_full_turn_refuses_the_next_command_without_changing_it() {
    let mut r = empty();
    r.accept(&(1, 0, vec![Command::Begin; 512]), false).unwrap();
    assert!(
        r.accept(&(1, 0, vec![Command::Pause { state: 1 }]), true)
            .is_err()
    );
    assert_eq!(r.orders[0].2.len(), 512);
    assert!(!r.paused);
    r.accept(&(2, 0, vec![Command::Begin]), false).unwrap();
}

#[test]
fn the_window_recording_rebuilds_byte_for_byte() {
    let Some(root) = std::env::var_os("RON_INSTALL") else {
        eprintln!("skipping window recording: set RON_INSTALL");
        return;
    };
    let file = PathBuf::from(std::env::var_os("HOME").unwrap())
        .join("ron-data/lab-experiments/2026-11-17-codex-live-recording/window-short.rcx");
    let logs = crate::world::gamelog_dir();
    if !logs.is_dir() {
        eprintln!("skipping window recording: no local recordings directory");
        return;
    }
    let roster = rondata::lobby::Roster::read(Path::new(&root)).unwrap();
    let choices = Choices::harness(&roster).unwrap();
    let mut r = Recording::open(
        &file.with_extension("test-only-unused"),
        &choices,
        &logs,
        empty().leader,
    )
    .unwrap();
    use crate::input::{Event, View};
    // Window-content pixels (the macOS title bar is outside the view).
    let events = [
        Event::Press { x: 503, y: 306 },
        Event::RightPress { x: 334, y: 374 },
        Event::Press { x: 562, y: 257 },
        Event::Key('c'),
    ];
    let mut world = crate::world::World::open_game(
        Path::new(&root),
        &logs,
        crate::world::Game::Page(Box::new(choices)),
    )
    .unwrap();
    let mut input = crate::input::Input::new(0);
    input.roster = world.names.roster.clone();
    world.runner.issue(&(0, 0, vec![r.leader.clone()]));
    world.run_to(30);
    for event in events {
        let scene = world.runner.scene(Some(0));
        let terrain = world.runner.terrain();
        let size = (960, 540);
        let view = View {
            matrix: crate::pinned_camera().to_screen_matrix(),
            screen: crate::pinned_screen(),
            size: Some(size),
            scene: &scene,
            terrain: &terrain,
            sim: Some(world.runner.sim()),
        };
        if let Some(order) = r.event(&mut input, size, 30, event, &view).unwrap() {
            world.runner.issue(&order);
        }
    }
    assert_eq!(r.orders.len(), 1, "both events share one turn");
    assert_eq!(r.orders[0].2.len(), 4, "{:?}", r.orders);
    world.run_to(427);
    r.accept(&(427, 0, vec![Command::Pause { state: 1 }]), true)
        .unwrap();
    let bytes = r.finish(world.runner.frame()).unwrap();
    assert_eq!(
        (bytes.len(), sim::mapgen::adler32(&bytes)),
        (59_249, 4_192_178_550),
        "{:?}",
        r.orders
    );
    if let Ok(expected) = std::fs::read(file) {
        assert!(
            bytes == expected,
            "every byte of the original-played window file"
        );
    }
}
