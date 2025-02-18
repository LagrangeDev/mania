use crate::core::event::prelude::*;
use crate::core::protos::message::CustomFace;
use crate::core::protos::service::oidb::{
    BytesPbReserveTroop, ClientMeta, CommonHead, ExtBizInfo, FileInfo, FileType, IPv4, MsgInfo,
    MultiMediaReqHead, NtGroupInfo, Ntv2RichMediaReq, Ntv2RichMediaResp, PicExtBizInfo,
    PttExtBizInfo, SceneInfo, UploadInfo, UploadReq, VideoExtBizInfo,
};
use crate::utility::random_gen::RandomGenerator;

#[derive(Debug, Default)]
pub struct ImageGroupUploadArgs {
    pub group_uin: u32,
    pub size: u32,
    pub md5: Bytes,
    pub sha1: Bytes,
    pub name: String,
    pub pic_type: u32,
    pub sub_type: u32,
    pub height: u32,
    pub width: u32,
    pub summary: String,
}

#[derive(Debug, Default)]
pub struct ImageGroupUploadRes {
    pub msg_info: MsgInfo,
    pub custom_face: CustomFace,
    pub u_key: Option<String>,
    pub ipv4s: Vec<IPv4>,
}

#[oidb_command(0x11c4, 100)]
#[derive(Debug, ServerEvent, Default)]
pub struct ImageGroupUploadEvent {
    pub req: ImageGroupUploadArgs,
    pub res: ImageGroupUploadRes,
}

impl ClientEvent for ImageGroupUploadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let req = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 1,
                    command: 100,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 1,
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
                            r#type: 1,
                            pic_format: self.req.pic_type,
                            video_format: 0,
                            voice_format: 0,
                        }),
                        width: self.req.width,
                        height: self.req.height,
                        time: 0,
                        original: 1,
                    }),
                    sub_file_type: 0,
                }],
                try_fast_upload_completed: true,
                srv_send_msg: false,
                client_random_id: RandomGenerator::rand_u64(),
                compat_q_msg_scene_type: 2,
                ext_biz_info: Some(dda!(ExtBizInfo {
                    pic: Some(dda!(PicExtBizInfo {
                        biz_type: self.req.sub_type,
                        text_summary: self.req.summary.to_owned(),
                        bytes_pb_reserve_troop: Some(dda!(BytesPbReserveTroop {
                            sub_type: self.req.sub_type,
                            text_summary: self.req.summary.to_owned(),
                        })),
                    })),
                    video: Some(dda!(VideoExtBizInfo {
                        bytes_pb_reserve: vec![],
                    })),
                    ptt: Some(dda!(PttExtBizInfo {
                        bytes_reserve: vec![],
                        bytes_pb_reserve: vec![],
                        bytes_general_flags: vec![],
                    })),
                })),
                client_seq: 0,
                no_need_compat_msg: false,
            }),
        });
        Ok(OidbPacket::new(0x11c4, 100, req.encode_to_vec(), false, true).to_binary())
    }

    fn parse(packet: Bytes, _: &Context) -> CEParseResult {
        let resp = OidbPacket::parse_into::<Ntv2RichMediaResp>(packet)?;
        let upload = resp
            .upload
            .ok_or_else(|| EventError::OtherError("Missing UploadResp".to_string()))?;
        let msg_info = upload
            .msg_info
            .ok_or_else(|| EventError::OtherError("Missing MsgInfo".to_string()))?;
        let custom_face = CustomFace::decode(Bytes::from(upload.compat_q_msg))?;
        let ipv4s = upload.i_pv4s;
        Ok(ClientResult::single(Box::new(dda!(Self {
            res: ImageGroupUploadRes {
                msg_info,
                custom_face,
                ipv4s,
                u_key: upload.u_key,
            }
        }))))
    }
}
