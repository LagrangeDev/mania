use crate::core::event::prelude::*;
use crate::core::protos::message::VideoFile;
use crate::core::protos::service::oidb::{
    C2cUserInfo, ClientMeta, CommonHead, ExtBizInfo, FileInfo, FileType, IPv4, MsgInfo,
    MultiMediaReqHead, Ntv2RichMediaReq, Ntv2RichMediaResp, PicExtBizInfo, PttExtBizInfo,
    SceneInfo, SubFileInfo, UploadInfo, UploadReq, VideoExtBizInfo,
};
use crate::utility::random_gen::RandomGenerator;

#[derive(Debug, Default)]
pub struct VideoC2CUploadArgs {
    pub uid: String,
    pub video_size: u32,
    pub video_md5: Bytes,
    pub video_sha1: Bytes,
    pub video_name: String,
    pub thumb_size: u32,
    pub thumb_md5: Bytes,
    pub thumb_sha1: Bytes,
    pub thumb_name: String,
    pub thumb_width: u32,
    pub thumb_height: u32,
    pub summary: String,
}

#[derive(Debug, Default)]
pub struct VideoC2CUploadRes {
    pub msg_info: MsgInfo,
    pub video_file: VideoFile,
    pub u_key: Option<String>,
    pub ipv4s: Vec<IPv4>,
    pub sub_file_info: Vec<SubFileInfo>,
}

#[oidb_command(0x11e9, 100)]
#[derive(Debug, ServerEvent, Default)]
pub struct VideoC2CUploadEvent {
    pub req: VideoC2CUploadArgs,
    pub res: VideoC2CUploadRes,
}

impl ClientEvent for VideoC2CUploadEvent {
    fn build(&self, _: &Context) -> CEBuildResult {
        let req = dda!(Ntv2RichMediaReq {
            req_head: Some(MultiMediaReqHead {
                common: Some(CommonHead {
                    request_id: 3,
                    command: 100,
                }),
                scene: Some(dda!(SceneInfo {
                    request_type: 2,
                    business_type: 2,
                    scene_type: 1,
                    c2c: Some(C2cUserInfo {
                        account_type: 2,
                        target_uid: self.req.uid.to_owned(),
                    })
                })),
                client: Some(ClientMeta { agent_type: 2 }),
            }),
            upload: Some(UploadReq {
                upload_info: vec![
                    UploadInfo {
                        file_info: Some(FileInfo {
                            file_size: self.req.video_size,
                            file_hash: hex::encode(self.req.video_md5.clone()),
                            file_sha1: hex::encode(self.req.video_sha1.clone()),
                            file_name: self.req.video_name.to_owned(),
                            r#type: Some(FileType {
                                r#type: 2,
                                pic_format: 0,
                                video_format: 0,
                                voice_format: 0,
                            }),
                            width: 0,
                            height: 0,
                            time: 0,
                            original: 0,
                        }),
                        sub_file_type: 0,
                    },
                    UploadInfo {
                        file_info: Some(FileInfo {
                            file_size: self.req.thumb_size,
                            file_hash: hex::encode(self.req.thumb_md5.clone()),
                            file_sha1: hex::encode(self.req.thumb_sha1.clone()),
                            file_name: self.req.thumb_name.to_owned(),
                            r#type: Some(FileType {
                                r#type: 1,
                                pic_format: 0,
                                video_format: 0,
                                voice_format: 0,
                            }),
                            width: self.req.thumb_width,
                            height: self.req.thumb_height,
                            time: 0,
                            original: 0,
                        }),
                        sub_file_type: 100,
                    }
                ],
                try_fast_upload_completed: true,
                srv_send_msg: false,
                client_random_id: RandomGenerator::rand_u64(),
                compat_q_msg_scene_type: 2,
                ext_biz_info: Some(dda!(ExtBizInfo {
                    pic: Some(dda!(PicExtBizInfo {
                        biz_type: 0,
                        text_summary: self.req.summary.to_owned(),
                    })),
                    video: Some(dda!(VideoExtBizInfo {
                        bytes_pb_reserve: vec![0x80, 0x01, 0x00],
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
        Ok(OidbPacket::new(0x11e9, 100, req.encode_to_vec(), false, true).to_binary())
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
        let video_file = VideoFile::decode(Bytes::from(upload.compat_q_msg))?;
        let ipv4s = upload.i_pv4s;
        Ok(ClientResult::single(Box::new(dda!(Self {
            res: VideoC2CUploadRes {
                msg_info,
                video_file,
                ipv4s,
                u_key: upload.u_key,
                sub_file_info
            }
        }))))
    }
}
