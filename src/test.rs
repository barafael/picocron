use test_case::test_case;

use super::*;
use futures::future::FutureExt;

use crate::seconds_till_next_offset;

#[test_case(5, 10 => 5)]
#[test_case(10, 5 => 55)]
fn should_calculate_seconds_till_next_offset(now: u8, offset: u8) -> u8 {
    seconds_till_next_offset(now, offset)
}

#[tokio::test(start_paused = true)]
async fn yields_first_id() {
    let mut schedule = Schedule::empty();
    schedule.add_timetable(10, vec![1]);
    let ten = schedule.next().await.expect("there is a job");
    assert_eq!(ten, 10);
}

#[tokio::test(start_paused = true)]
async fn remove_job() {
    let mut schedule = Schedule::empty();
    schedule.add_timetable(2, vec![1, 2, 3]);
    let two = schedule.next().await.expect("there is a job");
    assert_eq!(two, 2);
    let _removed = schedule.remove_timetable(&2).unwrap();
    assert!(schedule.next().now_or_never().unwrap().is_none());
}
