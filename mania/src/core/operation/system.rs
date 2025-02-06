use crate::core::business::BusinessHandle;
use crate::core::event::message::file_c2c_download::FileC2CDownloadEvent;
use crate::core::event::message::file_group_download::FileGroupDownloadEvent;
use crate::core::event::message::image_c2c_download::ImageC2CDownloadEvent;
use crate::core::event::message::image_group_download::ImageGroupDownloadEvent;
use crate::core::event::message::multi_msg_download::MultiMsgDownloadEvent;
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
}
