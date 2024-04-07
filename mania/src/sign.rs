use phf::{phf_set, Set};

use crate::Protocol;

mod linux;

static WHITELIST: Set<&'static str> = phf_set! {
    "trpc.o3.ecdh_access.EcdhAccess.SsoEstablishShareKey",
    "trpc.o3.ecdh_access.EcdhAccess.SsoSecureAccess",
    "trpc.o3.report.Report.SsoReport",
    "MessageSvc.PbSendMsg",
    "wtlogin.trans_emp",
    "wtlogin.login",
    "trpc.login.ecdh.EcdhService.SsoKeyExchange",
    "trpc.login.ecdh.EcdhService.SsoNTLoginPasswordLogin",
    "trpc.login.ecdh.EcdhService.SsoNTLoginEasyLogin",
    "trpc.login.ecdh.EcdhService.SsoNTLoginPasswordLoginNewDevice",
    "trpc.login.ecdh.EcdhService.SsoNTLoginEasyLoginUnusualDevice",
    "trpc.login.ecdh.EcdhService.SsoNTLoginPasswordLoginUnusualDevice",
    "OidbSvcTrpcTcp.0x11ec_1",
    "OidbSvcTrpcTcp.0x758_1", // create group
    "OidbSvcTrpcTcp.0x7c1_1",
    "OidbSvcTrpcTcp.0x7c2_5", // request friend
    "OidbSvcTrpcTcp.0x10db_1",
    "OidbSvcTrpcTcp.0x8a1_7", // request group
    "OidbSvcTrpcTcp.0x89a_0",
    "OidbSvcTrpcTcp.0x89a_15",
    "OidbSvcTrpcTcp.0x88d_0", // fetch group detail
    "OidbSvcTrpcTcp.0x88d_14",
    "OidbSvcTrpcTcp.0x112a_1",
    "OidbSvcTrpcTcp.0x587_74",
    "OidbSvcTrpcTcp.0x1100_1",
    "OidbSvcTrpcTcp.0x1102_1",
    "OidbSvcTrpcTcp.0x1103_1",
    "OidbSvcTrpcTcp.0x1107_1",
    "OidbSvcTrpcTcp.0x1105_1",
    "OidbSvcTrpcTcp.0xf88_1",
    "OidbSvcTrpcTcp.0xf89_1",
    "OidbSvcTrpcTcp.0xf57_1",
    "OidbSvcTrpcTcp.0xf57_106",
    "OidbSvcTrpcTcp.0xf57_9",
    "OidbSvcTrpcTcp.0xf55_1",
    "OidbSvcTrpcTcp.0xf67_1",
    "OidbSvcTrpcTcp.0xf67_5",
};

pub trait SignProvider: Send + Sync {
    fn sign(&self, cmd: &str, seq: u32, body: &[u8]) -> Option<SignResult> {
        if WHITELIST.contains(cmd) {
            self.sign_impl(cmd, seq, body)
        } else {
            None
        }
    }

    fn sign_impl(&self, cmd: &str, seq: u32, body: &[u8]) -> Option<SignResult>;
}

pub struct SignResult {
    pub software: Option<Vec<u8>>,
    pub token: Option<String>,
    pub signature: Vec<u8>,
}

pub fn default_sign_provider(protocol: Protocol) -> Box<dyn SignProvider> {
    match protocol {
        Protocol::Linux => Box::new(linux::LinuxSignProvider {}),
        _ => unimplemented!(),
    }
}
