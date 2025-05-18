use test_case::test_case;

use crate::seconds_till_next_offset;

#[test_case(5, 10 => 5)]
#[test_case(10, 5 => 55)]
fn should_calculate_seconds_till_next_offset(now: u8, offset: u8) -> u8 {
    seconds_till_next_offset(now, offset)
}
