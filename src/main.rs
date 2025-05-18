use chrono::{Local, Timelike};
use stream::Schedule;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let timetable1 = vec![3, 6, 9, 12, 18, 21, 24, 27, 33, 36, 39, 42];
    let timetable2 = vec![5, 10, 20, 25, 35, 40, 50, 55];
    let timetable3 = vec![15, 30, 45];

    let mut schedule = Schedule::empty();
    schedule.add_timetable("fizz", timetable1);
    schedule.add_timetable("buzz", timetable2);
    schedule.add_timetable("fizzbuzz", timetable3);

    // careful about empty stream map! May want to keep program alive when stream map is empty. Somebody else may insert in the future.
    // in a select!, that's less of an issue - just don't expect next().await on the map to be Some(_).
    while let Some(stream_id) = schedule.next().await {
        println!("{}@{}", stream_id, Local::now().second());
    }
}
