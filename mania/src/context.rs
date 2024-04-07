use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::Secp192k1;
use crate::key_store::KeyStore;
use crate::session::Session;
use crate::sign::SignProvider;
use crate::Protocol;

pub struct Context {
    pub app_info: &'static AppInfo,
    pub device: DeviceInfo,
    pub key_store: KeyStore,
    pub sign_provider: Box<dyn SignProvider>,
    pub crypto: Crypto,
    pub session: Session,
}

pub struct Crypto {
    pub secp: Secp192k1,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            secp: Secp192k1::new().unwrap(),
        }
    }
}

pub struct AppInfo {
    pub os: &'static str,
    pub vendor_os: &'static str,
    pub kernel: &'static str,
    pub base_version: &'static str,
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
        base_version: "3.1.1-11223",
        current_version: "3.1.2-13107",
        build_version: 13107,
        misc_bitmap: 32764,
        pt_version: "2.0.0",
        pt_os_version: 19,
        package_name: "com.tencent.qq",
        wt_login_sdk: "nt.wtlogin.0.0.1",
        package_sign: "V1_LNX_NQ_3.1.2-13107_RDM_B",
        app_id: 1600001615,
        sub_app_id: 537146866,
        app_id_qr_code: 13697054,
        app_client_version: 13172,
        main_sig_map: 169742560,
        sub_sig_map: 0,
        nt_login_type: 1,
    };

    pub const MAC_OS: AppInfo = AppInfo {
        os: "Mac",
        vendor_os: "mac",
        kernel: "Darwin",
        base_version: "6.9.17-12118",
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
        base_version: "9.9.1-15489",
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
