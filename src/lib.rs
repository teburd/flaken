use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::thread;

/// MonteFlake creates a configurable Snowflake ID generator where the epoch
/// and bitwidths may be adjusted to your liking from the defaults.
/// MonteFlake id generation will always increase in value from the time
/// its instantiated. No other guarantees are made. If a MonteFlake generator
/// is created after a clock moves back from other MonteFlake generated ids
/// conflicting ID values are entirely possible.
pub struct MonteFlake {
    id: u64,
    epoch: u64,
    bitwidths: (u64, u64, u64),
    seq: u64,
    start_ts: u64,
    start_instant: Instant,
    duration: u64,
}

trait AsMillis {
    fn as_millis(self) -> u64;
}

impl AsMillis for Duration {
    fn as_millis(self) -> u64 {
        self.as_secs()*1000 + (self.subsec_nanos() as u64)/1000000
    }
}

impl MonteFlake {
    /// Build a new monteflake with default options
    /// id: 0
    /// epoch: 2013-01-01T00:00:00Z in milliseconds since the unix epoch
    /// bitwidths (42 timestamp bits, 10 id bits, 12 sequence bits)
    pub fn new(id: u64) -> MonteFlake {
        let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let ts = since_unix.as_millis();
        let instant = Instant::now();
        MonteFlake {
            id: id,
            seq: 0,
            epoch: 1356998400000,
            bitwidths: (42, 10, 12),
            start_ts: ts,
            start_instant: instant,
            duration: 0,
        }
    }

    /// Set the epoch of a MonteFlake generator
    pub fn epoch(mut self, epoch: u64) -> MonteFlake {
        self.epoch = epoch;
        self
    }

    /// Set the id of a MonteFlake generator
    pub fn id(mut self, id: u64) -> MonteFlake {
        self.id = id;
        self
    }

    /// Set the bitwidths of a MonteFlake generator
    pub fn bitwidths(mut self, ts_bits: u64, id_bits: u64) -> MonteFlake {
        assert!(ts_bits + id_bits < 64);
        self.bitwidths = (ts_bits, id_bits, 64-(ts_bits+id_bits));
        self
    }

    /// generate the next id
    /// internally this updates at least the current sequence value, possibly
    /// the timestamp value if enough time has elapsed to matter
    pub fn next(&mut self) -> u64 {
        let duration = self.start_instant.elapsed().as_millis();
        if duration != self.duration {
            self.seq = 0;
        }
        let ts = self.start_ts + duration;
        let id = self.encode(ts, self.id, self.seq);
        self.duration = duration;
        self.seq += 1;
        id
    }

    /// encode into an id the current time, id of the generator (machine, node, shard), and
    /// sequence value
    /// the current time (ts) is the number of milliseconds passed since the unix epoch
    pub fn encode(&self, ts: u64, id: u64, seq: u64) -> u64 {
        let ts0 = ts - self.epoch;
        let (_, id_shift, seq_shift) = self.bitwidths;
        let ts_mask = bitmask(id_shift+seq_shift);
        let id_mask = bitmask(seq_shift) ^ ts_mask;
        let seq_mask = (bitmask(0) ^ ts_mask) ^ id_mask;
        ((ts0 << (id_shift + seq_shift)) & ts_mask) | ((id << seq_shift) & id_mask) | (seq & seq_mask)
    }

    /// decode from an encoded id the time, id of the generator, and sequence id
    /// the current time (ts) is the number of milliseconds passed since the unix epoch
    pub fn decode(&self, flake_id: u64) -> (u64, u64, u64) {
        let (_, id_shift, seq_shift) = self.bitwidths;
        let ts_mask = bitmask(id_shift+seq_shift);
        let id_mask = bitmask(seq_shift) ^ ts_mask;
        let seq_mask = (bitmask(0) ^ ts_mask) ^ id_mask;
        let ts = (flake_id & ts_mask) >> (id_shift+seq_shift);
        let id = (flake_id & id_mask) >> seq_shift;
        let seq = flake_id & seq_mask;
        (ts + self.epoch, id, seq)
    }
}

fn bitmask(left_shift: u64) -> u64 {
    0xFFFFFFFFFFFFFFFF << left_shift
}

#[test]
fn test_bitmask() {
    assert_eq!(bitmask(4), 0xFFFFFFFFFFFFFFF0);
    assert_eq!(bitmask(7), 0xFFFFFFFFFFFFFF80);
}

#[test]
fn test_encode_decode() {
    let flake = MonteFlake::new(0);
    let vals = (13+flake.start_ts,24,81);
    let id = flake.encode(vals.0, vals.1, vals.2); 
    assert_eq!(flake.decode(id), vals);
}

#[test]
fn test_next() {
    let new_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut flake = MonteFlake::new(0).epoch(new_epoch);
    let id0 = flake.next();
    let id1 = flake.next();
    let (ts0, id0, seq0) = flake.decode(id0);
    let (ts1, id1, seq1) = flake.decode(id1);
    assert!((ts0-new_epoch) < 1);
    assert_eq!(id0, 0);
    assert_eq!(seq0, 0);
    assert!((ts1-new_epoch) < 1);
    assert_eq!(id1, 0);
    assert_eq!(seq1, 1);
    let mut flake1 = flake.id(100);
    let id2 = flake1.next();
    let (ts2, id2, seq2) = flake1.decode(id2);
    assert!((ts2-new_epoch) < 1);
    assert_eq!(id2, 100);
    assert_eq!(seq2, 2);
    thread::sleep(Duration::from_millis(10));
    let id3 = flake1.next();
    let (ts3, id3, seq3) = flake1.decode(id3);
    assert!((ts3-new_epoch) >= 10);
    assert_eq!(id3, 100);
    assert_eq!(seq3, 0);
}
