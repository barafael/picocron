#[cfg(test)]
mod test;

use std::{pin::Pin, time::Duration};

use chrono::{Local, Timelike};
use futures::{Stream, StreamExt, stream::iter};
use tokio::time::{Instant, sleep_until};
use tokio_stream::StreamMap;

#[tokio::main]
async fn main() {
    let timetable1 = [1, 4, 12, 30, 40, 50];
    let timetable2 = [2, 5, 11, 29, 41, 51];

    let stream1 = make_stream(&timetable1);
    let stream2 = make_stream(&timetable2);

    let mut map = StreamMap::new();
    map.insert(1, stream1);
    map.insert(2, stream2);

    // careful about empty stream map! May want to keep program alive when stream map is empty. Somebody else may insert in the future.
    // in a select!, that's less of an issue - just don't expect next().await on the map to be Some(_).
    while let Some((stream_id, ())) = map.next().await {
        println!("{}@{}", stream_id, Local::now().second());
    }
}

/// offset list must be sorted.
fn make_stream(offsets: &[u8]) -> Pin<Box<dyn Stream<Item = ()> + '_>> {
    // skip offsets which have already passed in the current period.
    // assumption: stream is polled relatively soon after being created.
    // if not, it might take up to one period until reaching the first offset.
    let now = u8::try_from(Local::now().second()).expect("valid second");
    let preamble = offsets.iter().skip_while(move |offset| **offset < now);

    // after the preamble, repeat the offsets forever.
    let next_offset = preamble
        .chain(offsets.iter().cycle())
        // seconds till next offset.
        .map(|offset| {
            let now = u8::try_from(Local::now().second()).expect("valid second");
            let seconds = seconds_till_next_offset(now, *offset);
            let duration = Duration::from_secs(seconds as u64);
            Instant::now() + duration
        });

    // construct a stream which yields when the next offset is reached, repeating forever.
    iter(next_offset)
        // asynchronously wait until instant.
        .then(sleep_until)
        // box to implement traits of return type.
        .boxed()
}

/// Calculate seconds to given offset within current period.
///
/// `now` and `offset` shall be seconds of a minute (0..60).
/// In any case, the returned number within that range.
fn seconds_till_next_offset(now: u8, offset: u8) -> u8 {
    // projecting offset into the next minute makes sure there is no underflow
    // (given that `now` and `offset` are <60).
    ((offset + 60) - now) % 60
}
