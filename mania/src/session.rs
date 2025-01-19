use std::sync::atomic::{self, AtomicU16};
use tokio::sync::RwLock;

pub struct Session {
    pub stub: KeyCollection,
    pub qr_sign: RwLock<Option<QrSign>>,
    pub unusual_sign: RwLock<Option<Vec<u8>>>,
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
            qr_sign: RwLock::new(Some(QrSign {
                sign: [0; 24],
                string: String::new(),
                url: String::new(),
            })),
            unusual_sign: RwLock::new(None),
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
    pub random_key: RwLock<[u8; 16]>,
    pub tgtgt_key: RwLock<[u8; 16]>,
}

impl Default for KeyCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyCollection {
    pub fn new() -> Self {
        Self {
            random_key: RwLock::new([0; 16]),
            tgtgt_key: RwLock::new([0; 16]),
        }
    }
}

#[derive(Debug)]
pub struct QrSign {
    pub sign: [u8; 24],
    pub string: String,
    pub url: String,
}
