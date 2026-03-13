mod linux;

use crate::sign::linux::LinuxSignProvider;
use mania_core::core::context::Protocol;
use mania_core::core::sign::SignProvider;

pub fn default_sign_provider(protocol: Protocol, url: Option<String>) -> Box<dyn SignProvider> {
    match protocol {
        Protocol::Linux => Box::new(LinuxSignProvider { url }),
        _ => unimplemented!(),
    }
}
