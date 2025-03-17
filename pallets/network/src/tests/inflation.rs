use super::mock::*;
use crate::tests::test_utils::*;
use log::info;

//
//
//
//
//
//
//
// Inflation
//
//
//
//
//
//
//

#[test]
fn test_inflation() {
  new_test_ext().execute_with(|| {
    let inflation = Inflation::default();

    let mut last = inflation.total(0.0);

    for year in &[0.1, 0.5, 1.0, 50.0, 100.0] {
        let total = inflation.total(*year);
        assert_eq!(
          total,
          inflation.validator(*year) + inflation.foundation(*year)
        );
        assert!(total < last);
        assert!(total >= inflation.terminal);
        last = total;
    }
    assert_eq!(last, inflation.terminal);
  });
}