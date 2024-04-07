use crate::sign::{SignProvider, SignResult};

pub struct LinuxSignProvider {}

impl SignProvider for LinuxSignProvider {
    fn sign_impl(&self, _cmd: &str, _seq: u32, _body: &[u8]) -> Option<SignResult> {
        Some(SignResult {
            software: None,
            token: None,
            signature: vec![0; 35],
        })
    }
}
