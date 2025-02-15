use crate::ClientConfig;
use crate::core::crypto::consts::ECDH_256_PEER_LOGIN_KEY;
use crate::core::crypto::{Ecdh, P256};
use crate::core::key_store::KeyStore;
use crate::core::session::Session;
use crate::core::sign::SignProvider;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{fs, io};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Protocol {
    Windows = 0,
    MacOS = 1,
    Linux = 2,
}

pub struct Context {
    pub(crate) app_info: &'static AppInfo,
    pub(crate) device: DeviceInfo,
    pub(crate) key_store: KeyStore,
    pub(crate) sign_provider: Box<dyn SignProvider>,
    pub(crate) crypto: Crypto,
    pub(crate) session: Session,
    pub(crate) config: Arc<ClientConfig>,
}

pub struct Crypto {
    pub login_p256: P256,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            login_p256: P256::new(ECDH_256_PEER_LOGIN_KEY), // TODO: dynamic fetch (login) p256 pub key from tx's server
        }
    }
}

pub struct AppInfo {
    pub os: &'static str,
    pub vendor_os: &'static str,
    pub kernel: &'static str,
    pub current_version: &'static str,
    pub build_version: i32,
    pub misc_bitmap: i32,
    pub pt_version: &'static str,
    pub pt_os_version: i32,
    pub package_name: &'static str,
    pub wt_login_sdk: &'static str,
    pub package_sign: &'static str,
    pub app_id: i32,
    pub sub_app_id: i32,
    pub app_id_qr_code: i32,
    pub app_client_version: u16,
    pub main_sig_map: u32,
    pub sub_sig_map: u16,
    pub nt_login_type: u16,
}

impl AppInfo {
    pub const LINUX: AppInfo = AppInfo {
        os: "Linux",
        vendor_os: "linux",
        kernel: "Linux",
        current_version: "3.2.15-30366",
        build_version: 30366,
        misc_bitmap: 32764,
        pt_version: "2.0.0",
        pt_os_version: 19,
        package_name: "com.tencent.qq",
        wt_login_sdk: "nt.wtlogin.0.0.1",
        package_sign: "V1_LNX_NQ_3.2.15-30366_RDM_B",
        app_id: 1600001615,
        sub_app_id: 537258424,
        app_id_qr_code: 13697054,
        app_client_version: 30366,
        main_sig_map: 169742560,
        sub_sig_map: 0,
        nt_login_type: 1,
    };

    pub const MAC_OS: AppInfo = AppInfo {
        os: "Mac",
        vendor_os: "mac",
        kernel: "Darwin",
        current_version: "6.9.23-20139",
        build_version: 20139,
        misc_bitmap: 32764,
        pt_version: "2.0.0",
        pt_os_version: 23,
        package_name: "com.tencent.qq",
        wt_login_sdk: "nt.wtlogin.0.0.1",
        package_sign: "V1_MAC_NQ_6.9.23-20139_RDM_B",
        app_id: 1600001602,
        sub_app_id: 537200848,
        app_id_qr_code: 537200848,
        app_client_version: 13172,
        main_sig_map: 169742560,
        sub_sig_map: 0,
        nt_login_type: 5,
    };

    pub const WINDOWS: AppInfo = AppInfo {
        os: "Windows",
        vendor_os: "win32",
        kernel: "Windows_NT",
        current_version: "9.9.2-15962",
        build_version: 15962,
        misc_bitmap: 32764,
        pt_version: "2.0.0",
        pt_os_version: 23,
        package_name: "com.tencent.qq",
        wt_login_sdk: "nt.wtlogin.0.0.1",
        package_sign: "V1_WIN_NQ_9.9.2-15962_RDM_B",
        app_id: 1600001604,
        sub_app_id: 537138217,
        app_id_qr_code: 537138217,
        app_client_version: 13172,
        main_sig_map: 169742560,
        sub_sig_map: 0,
        nt_login_type: 5,
    };

    pub fn get(protocol: Protocol) -> &'static AppInfo {
        match protocol {
            Protocol::Windows => &AppInfo::WINDOWS,
            Protocol::Linux => &AppInfo::LINUX,
            Protocol::MacOS => &AppInfo::MAC_OS,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub uuid: Uuid,
    pub mac_address: Vec<u8>,
    pub device_name: String,
    pub system_kernel: String,
    pub kernel_version: String,
}

pub trait ExtendUuid {
    fn to_bytes(&self) -> &[u8];
}

impl ExtendUuid for Uuid {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes().as_ref()
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mac_address: Vec<u8> = (0..6).map(|_| rng.r#gen()).collect();
        Self {
            uuid: Uuid::new_v4(),
            mac_address,
            device_name: "Lagrange.Mania".to_string(),
            system_kernel: "Windows 10.0.19042".to_string(),
            kernel_version: "10.0.19042.0".to_string(),
        }
    }
}

impl DeviceInfo {
    pub fn load(file_path: &str) -> io::Result<DeviceInfo> {
        let data = fs::read_to_string(file_path)?;
        let device_info: DeviceInfo = serde_json::from_str(&data)?;
        Ok(device_info)
    }

    pub fn save(&self, file_path: &str) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }
}
