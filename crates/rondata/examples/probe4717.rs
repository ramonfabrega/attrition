// Item 4717's probe: row 83 replayed to 3716 (reverted whole).
fn main() {
    let inst = rondata::Install::new("/Users/rf-studio/code/fun/attrition/game");
    let loaded = rondata::load::load(&inst).unwrap();
    let rec = rondata::recgame::read(&std::env::args().nth(1).unwrap()).unwrap();
    let last: i32 = std::env::args().nth(2).map_or(3715, |s| s.parse().unwrap());
    let (info, players) = rondata::diff::sample::header_lobby(&rec);
    let info: Vec<_> = info.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let players: Vec<Vec<_>> = players
        .iter()
        .map(|p| p.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
        .collect();
    let start = rondata::diff::from_lobby(&loaded, &info, &players, sim::Tuning::RON);
    let (mut b, _) = rondata::diff::built_from_lobby(&loaded, start);
    let mut stream = rondata::input::Stream::pump(&rec);
    for f in 0..=last {
        if f == 3715 {
            for u in b.sim.units.iter().filter(|u| u.alive() && ((u.owner == 1 && (u.index == 24 || u.index == 38)) || (u.owner == 2 && u.index == 125))) {
                println!("unit {}/{} ti {} {:?} decoy {} health {}/{} full {} targeted {} orders {:?}", u.owner, u.index, u.type_index, u.pos, u.decoy, u.health, u.max_health, u.full, u.combat.targeted, u.orders);
            }
        }
        if f == 3715 && std::env::var_os("RON_PRE4717").is_some() {
            let find = |w: u8, o: i16| b.sim.units.iter().position(|u| u.alive() && u.owner == w && u.index == o).unwrap();
            let (a, d, r) = (find(2, 125), find(1, 38), find(1, 24));
            eprintln!("pre frame {}", b.sim.frame);
            for t in [d, r] {
                let v = b.sim.compare_target(sim::combat::Obj::Unit(a), sim::combat::Obj::Unit(t), true, false);
                eprintln!("pre compare_target(2/125, {}/{}) = {v}", b.sim.units[t].owner, b.sim.units[t].index);
            }
        }
        stream.apply(f, &mut b);
        b.tick();
    }
}
