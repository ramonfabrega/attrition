use super::*;

/// A one-site view: a list of tiles, every tile blocked and gatherable
/// unless listed in `shut`, the held tile gatherable per `held_ok`.
struct Mock {
    tiles: Vec<(i32, i32)>,
    shut: Vec<(i32, i32)>,
    held_ok: bool,
    wood: bool,
    draws: u32,
    flags: u32,
}

impl GatherTileView for Mock {
    fn site_recharge_flag(&mut self) -> bool {
        true
    }
    fn site_recharge_bump(&mut self) {}
    fn site_recharge_flag_set(&mut self) {}
    fn clear_group(&mut self) {}
    fn mark_leader(&mut self) {}
    fn stand(&mut self) {}
    fn order_tile_gatherable(&mut self, _tx: i32, _ty: i32) -> bool {
        self.held_ok
    }
    fn list_tile_gatherable(&mut self, x: i32, y: i32) -> bool {
        !self.shut.contains(&(x, y))
    }
    fn site_enter(&mut self) {}
    fn unit_type_index(&mut self) -> i32 {
        0
    }
    fn doober(&mut self) -> i32 {
        -1
    }
    fn remove_doober(&mut self, _id: i32) {}
    fn tiles_len(&mut self) -> i32 {
        self.tiles.len() as i32
    }
    fn site_mtn(&mut self) -> i32 {
        -1
    }
    fn mtn_tiny(&mut self) -> i32 {
        0
    }
    fn tile_at(&mut self, i: i32) -> (i32, i32) {
        self.tiles[i as usize]
    }
    fn tile_mask(&mut self, _x: i32, _y: i32) -> u16 {
        MASK_BLOCKED
    }
    fn building_tile(&mut self) -> (i32, i32) {
        (0, 0)
    }
    fn remove_tile(&mut self, x: i32, y: i32) {
        self.tiles.retain(|&t| t != (x, y));
    }
    fn push_tile(&mut self, x: i32, y: i32) {
        self.tiles.push((x, y));
    }
    fn rnd(&mut self) -> i32 {
        self.draws += 1;
        7
    }
    fn site_is_woodcutter(&mut self) -> bool {
        self.wood
    }
    fn or_unit_flags(&mut self, bits: u32) {
        self.flags |= bits;
    }
}

fn order(tx: i32, ty: i32) -> TileOrder {
    TileOrder {
        tx,
        ty,
        wait: -1,
        goto_build: true,
        dist_mod: 10,
        been_there: true,
    }
}

#[test]
fn a_stale_held_tile_stays_when_nothing_replaces_it() {
    let mut v = Mock {
        tiles: vec![(5, 5)],
        shut: vec![(5, 5)],
        held_ok: false,
        wood: true,
        draws: 0,
        flags: 0,
    };
    let mut o = order(3, 4);
    gather_tile_steps(&mut v, &mut o);
    assert_eq!(
        (o.tx, o.ty),
        (3, 4),
        "the held tile is written by a pick only"
    );
    assert_eq!(v.draws, 0, "no wait is rolled");
    assert!(o.goto_build && o.wait == -1);
}

#[test]
fn the_nearest_open_tile_moves_to_the_end_and_a_miner_never_counts_down() {
    let mut v = Mock {
        tiles: vec![(9, 9), (2, 2), (1, 1)],
        shut: vec![(1, 1)],
        held_ok: false,
        wood: false,
        draws: 0,
        flags: 0,
    };
    let mut o = order(-1, -1);
    gather_tile_steps(&mut v, &mut o);
    assert_eq!((o.tx, o.ty), (2, 2));
    assert_eq!(v.tiles, vec![(9, 9), (1, 1), (2, 2)]);
    assert_eq!(
        (o.wait, o.goto_build, v.flags),
        (MINER_WAIT, false, FLAG_TO_ORE)
    );
    assert_eq!(v.draws, 1);
}

#[test]
fn a_held_tile_that_is_still_open_is_kept_and_the_wait_rolled() {
    let mut v = Mock {
        tiles: vec![(9, 9)],
        shut: vec![],
        held_ok: true,
        wood: true,
        draws: 0,
        flags: 0,
    };
    let mut o = order(3, 4);
    gather_tile_steps(&mut v, &mut o);
    assert_eq!((o.tx, o.ty, o.wait), (3, 4, 407));
    assert_eq!(v.flags, FLAG_TO_WOOD);
}
