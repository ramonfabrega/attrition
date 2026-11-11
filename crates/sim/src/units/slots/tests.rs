use super::*;

/// `init_special`'s first free below the mark, else the mark; the
/// close walks the mark down past the free top (`docs/SCOUT.md` §16).
#[test]
fn a_slot_list_reuses_the_first_free_and_walks_its_mark_down() {
    let mut l = SlotList::default();
    assert_eq!((l.take(), l.take(), l.take()), (0, 1, 2));
    l.give_back(1);
    assert_eq!(l.mark, 3, "a hole below the top leaves the mark");
    assert_eq!(l.take(), 1, "the hole is taken first");
    l.give_back(2);
    l.give_back(1);
    assert_eq!(l.mark, 1, "the mark walks down past both");
    assert_eq!(l.take(), 1);
    assert_eq!(l.in_use.len(), 3, "the array never shrinks");
}

/// `add_hold_doober` takes the first fade-0 slot; a removed slot is
/// free on the 26th `inc_time`, and the free tail is trimmed
/// (`docs/ORDERS.md` §6.12).
#[test]
fn a_removed_hold_doober_frees_its_slot_on_the_26th_step() {
    let mut d = HoldDoobers::default();
    assert_eq!((d.add(), d.add(), d.add()), (0, 1, 2));
    d.remove(1);
    for _ in 0..25 {
        d.inc_time();
    }
    assert_eq!(d.fade[1], -5, "still fading after 25 steps");
    assert_eq!(d.add(), 3, "a fading slot is not free");
    d.inc_time();
    assert_eq!(d.fade[1], 0);
    assert_eq!(d.add(), 1, "free on the 26th");
    assert_eq!(d.fade[0], 255, "a held fade climbs to 255 and stays");
    d.remove(3);
    d.remove(2);
    for _ in 0..26 {
        d.inc_time();
    }
    assert_eq!(d.fade.len(), 2, "the free tail is trimmed");
    d.remove(9);
    assert_eq!(d.fade.len(), 2, "an index past the table writes nothing");
}
