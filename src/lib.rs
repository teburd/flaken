//! Flaken is a configurable Snowflake ID generator where the epoch
//! and bitwidths may be adjusted to your liking from the defaults.
//! ID generation from an instantiated generator will always increase in value.
//! If a ID generator is created after a clock moves back from previously
//! created IDs conflicting ID values are possible, otherwise clock changes do
//! not affect ID generation.
//!
//! # Encode and decode example
//!
//! ```
//! use std::time;
//! use flaken::Flaken;
//!
//! let ts = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
//! let ts_ms = ts.as_secs()*1000 + (ts.subsec_nanos() as u64)/1000000;
//! let mf = Flaken::default();
//! let id = mf.encode(ts_ms, 10, 100);
//! let (ts0, node0, seq0) = mf.decode(id);
//! assert_eq!(ts0, ts_ms);
//! assert_eq!(node0, 10);
//! assert_eq!(seq0, 100);
//! ```
//!
//! # ID generation example
//!
//! ```
//! use flaken::Flaken;
//!
//! let mut flake = Flaken::default().node(1).epoch(0).bitwidths(40, 10);
//! let id0 = flake.next();
//! let (ts0, node0, seq0) = flake.decode(id0);
//! assert!(ts0 > 0);
//! assert_eq!(node0, 1);
//! assert_eq!(seq0, 0);
//! assert_eq!(flake.encode(ts0, node0, seq0), id0);
//! ```

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::thread;


/// Flaken ID generator, encoder, and decoder
pub struct Flaken {
    node: u64,
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

impl Flaken {
    /// Build a new flake id with the given node id and other default options
    /// node: 0
    /// epoch: 2013-01-01T00:00:00Z in milliseconds since the unix epoch
    /// bitwidths (42 timestamp bits, 10 id bits, 12 sequence bits)
    pub fn default() -> Flaken {
        let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let ts = since_unix.as_millis();
        let instant = Instant::now();
        Flaken {
            node: 0,
            seq: 0,
            epoch: 1356998400000,
            bitwidths: (42, 10, 12),
            start_ts: ts,
            start_instant: instant,
            duration: 0,
        }
    }

    /// Set the epoch of a Flaken generator
    pub fn epoch(mut self, epoch: u64) -> Flaken {
        self.epoch = epoch;
        self
    }

    /// Set the node id of a Flaken generator
    pub fn node(mut self, node: u64) -> Flaken {
        self.node = node;
        self
    }

    /// Set the bitwidths of a Flaken generator
    pub fn bitwidths(mut self, ts_bits: u64, node_bits: u64) -> Flaken {
        assert!(ts_bits + node_bits < 64);
        self.bitwidths = (ts_bits, node_bits, 64-(ts_bits+node_bits));
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
        let id = self.encode(ts, self.node, self.seq);
        self.duration = duration;
        self.seq += 1;
        id
    }

    /// Encode into a flake id the given id, current time, and sequence value
    ///
    /// The current time (ts) is the number of milliseconds passed since the unix epoch
    pub fn encode(&self, ts: u64, node: u64, seq: u64) -> u64 {
        assert!(ts >= self.epoch);
        let ts0 = ts - self.epoch;
        let (_, node_shift, seq_shift) = self.bitwidths;
        let ts_mask = bitmask(node_shift+seq_shift);
        let node_mask = bitmask(seq_shift) ^ ts_mask;
        let seq_mask = (bitmask(0) ^ ts_mask) ^ node_mask;
        ((ts0 << (node_shift + seq_shift)) & ts_mask) | ((node << seq_shift) & node_mask) | (seq & seq_mask)
    }

    /// Decode from an encoded id the timestamp, node id, and sequence id
    //
    /// The current time (ts) is the number of milliseconds passed since the unix epoch
    ///
    ///
    pub fn decode(&self, id: u64) -> (u64, u64, u64) {
        let (_, node_shift, seq_shift) = self.bitwidths;
        let ts_mask = bitmask(node_shift+seq_shift);
        let node_mask = bitmask(seq_shift) ^ ts_mask;
        let seq_mask = (bitmask(0) ^ ts_mask) ^ node_mask;
        let ts = (id & ts_mask) >> (node_shift+seq_shift);
        let node = (id & node_mask) >> seq_shift;
        let seq = id & seq_mask;
        (ts + self.epoch, node, seq)
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
    let flake = Flaken::default();
    let vals = (13+flake.start_ts,24,81);
    let id = flake.encode(vals.0, vals.1, vals.2);
    assert_eq!(flake.decode(id), vals);
}

#[test]
fn test_next() {
    let new_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut flake = Flaken::default().epoch(new_epoch);
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
    let mut flake1 = flake.node(100);
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
