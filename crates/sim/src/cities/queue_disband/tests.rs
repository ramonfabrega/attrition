use crate::cities::production::Queue;

#[test]
fn a_disband_entry_is_counted_and_costs_nothing() {
    let mut q = Queue::new(5);
    q.push(3, &[0; crate::economy::RESOURCES]);
    let s = q.push_disband();
    assert_eq!(s, 1);
    assert_eq!(q.count_disband(), 1);
    assert!(q.items[1].disband && q.items[1].tech.is_none());
    assert_eq!(q.items[1].paid().count(), 0);
}
