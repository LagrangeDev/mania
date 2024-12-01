use std::sync::atomic::{self, AtomicU16};

use arc_swap::ArcSwapOption;

pub struct Session {
    pub stub: KeyCollection,
    pub qr_sign: ArcSwapOption<QrSign>,
    pub unusual_sign: Option<Vec<u8>>,
    // TODO: other fields
    sequence: AtomicU16,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            stub: KeyCollection::new(),
            qr_sign: ArcSwapOption::default(),
            unusual_sign: None,
            sequence: AtomicU16::new(0),
        }
    }

    pub fn next_sequence(&self) -> u16 {
        self.sequence.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub fn set_sequence(&self, seq: u16) {
        self.sequence.store(seq, atomic::Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct KeyCollection {
    pub random_key: [u8; 16],
    pub tgt_key: [u8; 16],
}

impl Default for KeyCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyCollection {
    pub fn new() -> Self {
        Self {
            random_key: rand::random(),
            tgt_key: [0; 16],
        }
    }
}

#[derive(Debug)]
pub struct QrSign {
    pub sign: [u8; 24],
    pub string: String,
    pub url: String,
}
