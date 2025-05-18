use std::{pin::Pin, time::Duration};

use chrono::{DateTime, Local, Timelike};
use futures::{Stream, StreamExt, stream::iter};
use tokio::time::{Instant, sleep_until};
use tokio_stream::StreamMap;

#[cfg(test)]
mod test;

pub struct Schedule<K> {
    map: StreamMap<K, Pin<Box<dyn Stream<Item = ()>>>>,
}

impl<K> Schedule<K>
where
    K: std::hash::Hash + Eq,
{
    pub fn empty() -> Self {
        Schedule {
            map: StreamMap::new(),
        }
    }

    pub fn add_timetable(&mut self, id: K, offsets: Vec<u8>) {
        let stream = make_stream(offsets);
        self.map.insert(id, stream);
    }

    pub fn remove_timetable(&mut self, id: &K) -> Option<Pin<Box<dyn Stream<Item = ()>>>> {
        self.map.remove(id)
    }
}

impl<K> Stream for Schedule<K>
where
    K: Clone + Unpin,
{
    type Item = K;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.map.poll_next_unpin(cx).map(|o| o.map(|(id, ())| id))
    }
}

/// Helper function to create a stream which always yields at the next instant where an offset is reached.
///
/// 1. All offsets in the current period which have already passed are discarded.
/// 2. The offsets for each period are repeated, ad infinitum.
///    For each offset, the "time-until" is calculated based on the current second within the period.
///
/// Because of lazy evaluation, all calculation for the next period
/// happens when the stream is polled.
/// If a trigger time is overrun, all triggers until the same trigger time in the next period will be discarded.
fn make_stream(offsets: Vec<u8>) -> Pin<Box<dyn Stream<Item = ()>>> {
    let next_offset = make_iterator(offsets, Local::now).map(|duration| Instant::now() + duration);

    // construct a stream which yields when the next offset is reached, repeating forever.
    iter(next_offset)
        // asynchronously wait until instant.
        .then(sleep_until)
        // box to implement traits of return type.
        .boxed()
}

fn make_iterator(
    mut offsets: Vec<u8>,
    local: impl Fn() -> DateTime<Local>,
) -> impl Iterator<Item = Duration> {
    offsets.sort_unstable();
    // skip offsets which have already passed in the current period.
    // assumption: stream is polled relatively soon after being created.
    // if not, it might take up to one period until reaching the first offset.
    let now = u8::try_from(local().second()).expect("valid second");
    let preamble = offsets
        .clone()
        .into_iter()
        .skip_while(move |offset| *offset < now);

    // after the preamble, repeat the offsets forever.
    preamble
        .chain(offsets.into_iter().cycle())
        // seconds till next offset.
        .map(move |offset| {
            let now = u8::try_from(local().second()).expect("valid second");
            let seconds = seconds_till_next_offset(now, offset);
            Duration::from_secs(seconds as u64)
        })
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

#[cfg(test)]
mod test_utility_functions {
    use crate::make_iterator;
    use chrono::DateTime;
    use std::time::Duration;

    #[test]
    fn makes_iterator_with_increasing_offset_durations() {
        let mut offsets = make_iterator(vec![1, 2, 3], DateTime::default);
        let one = offsets.next().unwrap();
        let two = offsets.next().unwrap();
        let three = offsets.next().unwrap();
        assert_eq!(one, Duration::from_secs(1));
        assert_eq!(two, Duration::from_secs(2));
        assert_eq!(three, Duration::from_secs(3));
    }
}
