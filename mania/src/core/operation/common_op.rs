use crate::core::business::BusinessHandle;
use crate::core::event::message::file_c2c_download::FileC2CDownloadEvent;
use crate::core::event::message::file_group_download::FileGroupDownloadEvent;
use crate::core::event::message::image_c2c_download::ImageC2CDownloadEvent;
use crate::core::event::message::image_group_download::ImageGroupDownloadEvent;
use crate::core::event::message::multi_msg_download::MultiMsgDownloadEvent;
use crate::core::event::message::record_c2c_download::RecordC2CDownloadEvent;
use crate::core::event::message::record_group_download::RecordGroupDownloadEvent;
use crate::core::event::message::video_c2c_download::VideoC2CDownloadEvent;
use crate::core::event::message::video_group_download::VideoGroupDownloadEvent;
use crate::core::event::system::fetch_rkey::FetchRKeyEvent;
use crate::core::event::{downcast_event, downcast_mut_event};
use crate::core::protos::service::oidb::IndexNode;
use crate::dda;
use crate::message::chain::MessageChain;
use std::sync::Arc;

impl BusinessHandle {
    pub async fn fetch_rkey(self: &Arc<Self>) -> crate::Result<()> {
        let mut fetch_event = FetchRKeyEvent {};
        let res = self.send_event(&mut fetch_event).await?;
        tracing::info!("FetchRKeyEvent: {:?}", res);
        Ok(()) // TODO:
    }

    pub async fn download_group_image(
        self: &Arc<Self>,
        group_uin: u32,
        index_node: IndexNode,
    ) -> crate::Result<String> {
        let mut fetch_event = dda!(ImageGroupDownloadEvent {
            group_uin,
            index_node,
        });
        let res = self.send_event(&mut fetch_event).await?;
        let event: &ImageGroupDownloadEvent = downcast_event(&res).unwrap();
        Ok(event.image_url.clone())
    }

    pub async fn download_c2c_image(
        self: &Arc<Self>,
        index_node: IndexNode,
    ) -> crate::Result<String> {
        let uid = self
            .context
            .key_store
            .uid
            .load()
            .as_ref()
            .map(|arc| arc.as_ref().clone())
            .unwrap();
        let mut fetch_event = dda!(ImageC2CDownloadEvent {
            self_uid: uid,
            index_node,
        });
        let res = self.send_event(&mut fetch_event).await?;
        let event: &ImageC2CDownloadEvent = downcast_event(&res).unwrap();
        Ok(event.image_url.clone())
    }

    pub async fn multi_msg_download(
        self: &Arc<Self>,
        uid: String,
        res_id: String,
    ) -> crate::Result<Option<Vec<MessageChain>>> {
        let mut fetch_event = dda!(MultiMsgDownloadEvent {
            uid: Some(uid),
            res_id: Some(res_id),
        });
        let mut res = self.send_event(&mut fetch_event).await?;
        let event: &mut MultiMsgDownloadEvent = downcast_mut_event(&mut *res).unwrap();
        Ok(event.chains.take())
    }

    pub(crate) async fn download_group_file(
        self: &Arc<Self>,
        group_uin: u32,
        file_id: String,
    ) -> crate::Result<String> {
        let mut event = dda!(FileGroupDownloadEvent { group_uin, file_id });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut FileGroupDownloadEvent = downcast_mut_event(&mut *res).unwrap();
        Ok(event.file_url.to_owned())
    }

    pub(crate) async fn download_c2c_file(
        self: &Arc<Self>,
        file_uuid: Option<String>,
        file_hash: Option<String>,
        sender_uid: Option<String>,
    ) -> crate::Result<String> {
        let self_uid = self
            .context
            .key_store
            .uid
            .load()
            .as_ref()
            .map(|arc| arc.as_ref().clone());
        let mut event = dda!(FileC2CDownloadEvent {
            file_uuid,
            file_hash,
            sender_uid,
            receiver_uid: self_uid,
        });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut FileC2CDownloadEvent = downcast_mut_event(&mut *res).unwrap();
        Ok(event.file_url.to_owned())
    }

    // TODO: Should parameter requests use an enum?
    pub(crate) async fn download_group_record_by_node(
        self: &Arc<Self>,
        group_uin: u32,
        node: Option<IndexNode>,
    ) -> crate::Result<String> {
        self.download_group_record_inner(group_uin, node, None)
            .await
    }

    pub(crate) async fn download_group_record_by_uuid(
        self: &Arc<Self>,
        group_uin: u32,
        audio_uuid: Option<String>,
    ) -> crate::Result<String> {
        self.download_group_record_inner(group_uin, None, audio_uuid)
            .await
    }

    async fn download_group_record_inner(
        self: &Arc<Self>,
        group_uin: u32,
        node: Option<IndexNode>,
        audio_uuid: Option<String>,
    ) -> crate::Result<String> {
        let mut event = dda!(RecordGroupDownloadEvent {
            group_uin,
            node,
            file_uuid: audio_uuid.unwrap_or_default(),
        });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut RecordGroupDownloadEvent = downcast_mut_event(&mut *res).unwrap();
        Ok(event.audio_url.to_owned())
    }

    // TODO: Should parameter requests use an enum?
    pub(crate) async fn download_c2c_record_by_node(
        self: &Arc<Self>,
        node: Option<IndexNode>,
    ) -> crate::Result<String> {
        self.download_c2c_record_inner(node, None).await
    }

    pub(crate) async fn download_c2c_record_by_uuid(
        self: &Arc<Self>,
        audio_uuid: Option<String>,
    ) -> crate::Result<String> {
        self.download_c2c_record_inner(None, audio_uuid).await
    }

    async fn download_c2c_record_inner(
        self: &Arc<Self>,
        node: Option<IndexNode>,
        audio_uuid: Option<String>,
    ) -> crate::Result<String> {
        let self_uid = self
            .context
            .key_store
            .uid
            .load()
            .as_ref()
            .map(|arc| arc.as_ref().clone());
        let mut event = dda!(RecordC2CDownloadEvent {
            self_uid: self_uid.expect("Missing self_uid"),
            node,
            file_uuid: audio_uuid.unwrap_or_default(),
        });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut RecordC2CDownloadEvent = downcast_mut_event(&mut *res).unwrap();
        Ok(event.audio_url.to_owned())
    }

    pub async fn download_video(
        self: &Arc<Self>,
        self_uid: &str,
        file_name: &str,
        file_md5: &str,
        file_sha1: Option<String>,
        node: Option<IndexNode>,
        is_group: bool,
    ) -> crate::Result<String> {
        let mut event = dda!(VideoC2CDownloadEvent {
            self_uid: self_uid.to_string(),
            file_name: file_name.to_string(),
            file_md5: file_md5.to_string(),
            file_sha1,
            node,
            is_group,
        });
        let res = self.send_event(&mut event).await?;
        let event: &VideoC2CDownloadEvent = downcast_event(&res).unwrap();
        Ok(event.video_url.clone())
    }

    pub async fn download_group_video(
        self: &Arc<Self>,
        group_uin: u32,
        file_name: &str,
        file_md5: &str,
        file_sha1: Option<String>,
        node: Option<IndexNode>,
    ) -> crate::Result<String> {
        let mut event = dda!(VideoGroupDownloadEvent {
            group_uin,
            self_uid: self
                .context
                .key_store
                .uid
                .load()
                .as_ref()
                .unwrap()
                .as_ref()
                .clone(),
            file_name: file_name.to_string(),
            file_md5: file_md5.to_string(),
            file_sha1,
            node,
        });
        let res = self.send_event(&mut event).await?;
        let event: &VideoGroupDownloadEvent = downcast_event(&res).unwrap();
        Ok(event.video_url.clone())
    }
}
