/// The record counts only between `start` and `take`, keeps each
/// hook's first state, and sorts by it.
#[test]
fn a_hook_keeps_its_first_state_and_its_count() {
    super::hit("before");
    super::start();
    super::at(7);
    crate::seam!("late");
    super::at(3);
    crate::seam!("early");
    super::at(9);
    crate::seam!("late");
    let got = super::take();
    assert_eq!(
        got,
        vec![
            (
                "sim::core::seam::tests::early",
                super::Entered { first: 3, count: 1 }
            ),
            (
                "sim::core::seam::tests::late",
                super::Entered { first: 7, count: 2 }
            ),
        ]
    );
    crate::seam!("after");
    assert!(super::take().is_empty());
}
