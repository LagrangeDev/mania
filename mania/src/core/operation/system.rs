use crate::core::business::BusinessHandle;
use crate::core::event::downcast_event;
use crate::core::event::message::image_c2c_download::ImageC2CDownloadEvent;
use crate::core::event::message::image_group_download::ImageGroupDownloadEvent;
use crate::core::event::system::fetch_rkey::FetchRKeyEvent;
use crate::core::protos::service::oidb::IndexNode;
use crate::dda;
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
}
