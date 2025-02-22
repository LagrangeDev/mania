use crate::core::event::prelude::*;
use crate::core::protos::message::RichText;
use crate::core::protos::service::oidb::{
    ClientMeta, CommonHead, ExtBizInfo, FileInfo, FileType, IPv4, MsgInfo, MultiMediaReqHead,
    NtGroupInfo, Ntv2RichMediaReq, Ntv2RichMediaResp, PicExtBizInfo, PttExtBizInfo, SceneInfo,
    SubFileInfo, UploadInfo, UploadReq, VideoExtBizInfo,
};
use crate::utility::random_gen::RandomGenerator;

#[derive(Debug, Default)]
pub struct RecordGroupUploadArgs {
    pub group_uin: u32,
    pub size: u32,
    pub length: u32,
    pub md5: Bytes,
    pub sha1: Bytes,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct RecordGroupUploadRes {
    pub msg_info: MsgInfo,
    pub rich_text: RichText,
    pub sub_file_info: Vec<SubFileInfo>,
    pub u_key: Option<String>,
    pub ipv4s: Vec<IPv4>,
}

#[oidb_command(0x126e, 100)]
#[derive(Debug, ServerEvent, Default)]
pub struct RecordGroupUploadEvent {
    pub req: RecordGroupUploadArgs,
    pub res: RecordGroupUploadRes,
}

impl ClientEvent for RecordGroupUploadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let req = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 100,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 3,
                    scene_type: 2,
                    group: Some(NtGroupInfo {
                        group_uin: self.req.group_uin,
                    }),
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            upload: Some(UploadReq {
                upload_info: vec![UploadInfo {
                    file_info: Some(FileInfo {
                        file_size: self.req.size,
                        file_hash: hex::encode(self.req.md5.clone()),
                        file_sha1: hex::encode(self.req.sha1.clone()),
                        file_name: self.req.name.to_owned(),
                        r#type: Some(FileType {
                            r#type: 3,
                            pic_format: 0,
                            video_format: 0,
                            voice_format: 1,
                        }),
                        width: 0,
                        height: 0,
                        time: self.req.length,
                        original: 0,
                    }),
                    sub_file_type: 0,
                }],
                try_fast_upload_completed: true,
                srv_send_msg: false,
                client_random_id: RandomGenerator::rand_u64(),
                compat_q_msg_scene_type: 2,
                ext_biz_info: Some(dda!(ExtBizInfo {
                    pic: Some(dda!(PicExtBizInfo {
                        text_summary: "".to_string(),
                    })),
                    video: Some(dda!(VideoExtBizInfo {
                        bytes_pb_reserve: vec![],
                    })),
                    ptt: Some(dda!(PttExtBizInfo {
                        bytes_reserve: vec![],
                        bytes_pb_reserve: vec![0x08, 0x00, 0x38, 0x00],
                        bytes_general_flags: vec![
                            0x9a, 0x01, 0x07, 0xaa, 0x03, 0x04, 0x08, 0x08, 0x12, 0x00
                        ],
                    })),
                })),
                client_seq: 0,
                no_need_compat_msg: false,
            }),
        });
        Ok(OidbPacket::new(0x126e, 100, req.encode_to_vec(), false, true).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let resp = OidbPacket::parse_into::<Ntv2RichMediaResp>(packet)?;
        let upload = resp
            .upload
            .ok_or_else(|| EventError::OtherError("Missing UploadResp".to_string()))?;
        let msg_info = upload
            .msg_info
            .ok_or_else(|| EventError::OtherError("Missing MsgInfo".to_string()))?;
        let sub_file_info = upload.sub_file_infos;
        let rich_text = RichText::decode(Bytes::from(upload.compat_q_msg))?;
        let ipv4s = upload.i_pv4s;
        Ok(ClientResult::single(Box::new(dda!(Self {
            res: RecordGroupUploadRes {
                msg_info,
                rich_text,
                sub_file_info,
                ipv4s,
                u_key: upload.u_key,
            }
        }))))
    }
}
