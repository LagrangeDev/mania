use digest::Digest;
use sha1::Sha1;

#[derive(Default)]
pub struct TriSha1 {
    hasher: Sha1,
    offset: u64,
    file_size: u64,
    ranges: Vec<(u64, u64)>,
}

impl TriSha1 {
    pub fn new(file_size: u64) -> Self {
        let ranges = match file_size {
            // < 30M
            0..31457280u64 => vec![(0, file_size)],
            // >= 30M
            _ => vec![
                (0, 10485759u64),
                ((file_size >> 1) - 5242880, (file_size >> 1) + 5242879),
                (file_size - 10485760, file_size - 1),
            ],
        };
        Self {
            file_size,
            ranges,
            ..Default::default()
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        let start = self.offset;
        let end = start + data.len() as u64;
        for &(rs, re) in &self.ranges {
            let from = start.max(rs);
            let to = end.min(re + 1);
            if from < to {
                let slice = &data[(from - start) as usize..(to - start) as usize];
                Digest::update(&mut self.hasher, slice);
            }
        }
        self.offset = end;
    }

    pub fn finalize(mut self) -> [u8; 20] {
        let size_bytes = self.file_size.to_ne_bytes();
        Digest::update(&mut self.hasher, size_bytes);
        self.hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::extensions::HexString;

    fn make_data(size: usize) -> Vec<u8> {
        let pat = b"114514";
        let mut data = Vec::with_capacity(size);
        while data.len() < size {
            let rem = size - data.len();
            let n = rem.min(pat.len());
            data.extend_from_slice(&pat[..n]);
        }
        data
    }

    fn inner_test(size: u64) -> String {
        let mut hasher = TriSha1::new(size);
        let data = make_data(size as usize);
        for chunk in data.chunks(8192) {
            hasher.update(chunk);
        }
        hasher.finalize().hex()
    }

    #[test]
    fn tri_sha1_small() {
        let size = 5 * 1024 * 1024;
        assert_eq!(inner_test(size), "8f155d8b6b1a9c196597b01d142f0d046ad44923")
    }

    #[test]
    fn tri_sha1_mid() {
        let size = 15 * 1024 * 1024;
        assert_eq!(inner_test(size), "4748e8fdf867ea92d4d636e7312585ed93c80d0d")
    }

    #[test]
    fn tri_sha1_large() {
        let size = 50 * 1024 * 1024;
        assert_eq!(inner_test(size), "90d9a46f42f48dcee04f7390d4748e2a4c11d099")
    }
}
