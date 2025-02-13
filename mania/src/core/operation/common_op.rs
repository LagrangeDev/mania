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
use crate::core::event::system::fetch_filtered_group_request::FetchFilteredGroupRequestsEvent;
use crate::core::event::system::fetch_group_requests::FetchGroupRequestsEvent;
use crate::core::event::system::fetch_rkey::FetchRKeyEvent;
use crate::core::event::system::fetch_user_info::FetchUserInfoEvent;
use crate::core::event::{downcast_major_event, downcast_mut_major_event};
use crate::core::protos::service::oidb::IndexNode;
use crate::entity::bot_group_request::BotGroupRequest;
use crate::message::chain::MessageChain;
use crate::{ManiaError, ManiaResult, dda};
use futures::future::join_all;
use std::sync::Arc;
use tokio::join;

impl BusinessHandle {
    pub async fn fetch_rkey(self: &Arc<Self>) -> ManiaResult<()> {
        let mut fetch_event = FetchRKeyEvent {};
        let res = self.send_event(&mut fetch_event).await?;
        tracing::info!("FetchRKeyEvent: {:?}", res);
        Ok(()) // TODO:
    }

    pub async fn download_group_image(
        self: &Arc<Self>,
        group_uin: u32,
        index_node: IndexNode,
    ) -> ManiaResult<String> {
        let mut fetch_event = dda!(ImageGroupDownloadEvent {
            group_uin,
            index_node,
        });
        let res = self.send_event(&mut fetch_event).await?;
        let event: &ImageGroupDownloadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.image_url.clone())
    }

    pub async fn download_c2c_image(
        self: &Arc<Self>,
        index_node: IndexNode,
    ) -> ManiaResult<String> {
        let uid = self
            .context
            .key_store
            .uid
            .load()
            .as_ref()
            .map(|arc| arc.as_ref().clone())
            .expect("Missing self_uid");
        let mut fetch_event = dda!(ImageC2CDownloadEvent {
            self_uid: uid,
            index_node,
        });
        let res = self.send_event(&mut fetch_event).await?;
        let event: &ImageC2CDownloadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.image_url.clone())
    }

    pub async fn multi_msg_download(
        self: &Arc<Self>,
        uid: String,
        res_id: String,
    ) -> ManiaResult<Option<Vec<MessageChain>>> {
        let mut fetch_event = dda!(MultiMsgDownloadEvent {
            uid: Some(uid),
            res_id: Some(res_id),
        });
        let mut res = self.send_event(&mut fetch_event).await?;
        let event: &mut MultiMsgDownloadEvent =
            downcast_mut_major_event(&mut res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.chains.take())
    }

    pub(crate) async fn download_group_file(
        self: &Arc<Self>,
        group_uin: u32,
        file_id: String,
    ) -> ManiaResult<String> {
        let mut event = dda!(FileGroupDownloadEvent { group_uin, file_id });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut FileGroupDownloadEvent =
            downcast_mut_major_event(&mut res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.file_url.to_owned())
    }

    pub(crate) async fn download_c2c_file(
        self: &Arc<Self>,
        file_uuid: Option<String>,
        file_hash: Option<String>,
        sender_uid: Option<String>,
    ) -> ManiaResult<String> {
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
        let event: &mut FileC2CDownloadEvent =
            downcast_mut_major_event(&mut res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.file_url.to_owned())
    }

    // TODO: Should parameter requests use an enum?
    pub(crate) async fn download_group_record_by_node(
        self: &Arc<Self>,
        group_uin: u32,
        node: Option<IndexNode>,
    ) -> ManiaResult<String> {
        self.download_group_record_inner(group_uin, node, None)
            .await
    }

    pub(crate) async fn download_group_record_by_uuid(
        self: &Arc<Self>,
        group_uin: u32,
        audio_uuid: Option<String>,
    ) -> ManiaResult<String> {
        self.download_group_record_inner(group_uin, None, audio_uuid)
            .await
    }

    async fn download_group_record_inner(
        self: &Arc<Self>,
        group_uin: u32,
        node: Option<IndexNode>,
        audio_uuid: Option<String>,
    ) -> ManiaResult<String> {
        let mut event = dda!(RecordGroupDownloadEvent {
            group_uin,
            node,
            file_uuid: audio_uuid.unwrap_or_default(),
        });
        let mut res = self.send_event(&mut event).await?;
        let event: &mut RecordGroupDownloadEvent =
            downcast_mut_major_event(&mut res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.audio_url.to_owned())
    }

    // TODO: Should parameter requests use an enum?
    pub(crate) async fn download_c2c_record_by_node(
        self: &Arc<Self>,
        node: Option<IndexNode>,
    ) -> ManiaResult<String> {
        self.download_c2c_record_inner(node, None).await
    }

    pub(crate) async fn download_c2c_record_by_uuid(
        self: &Arc<Self>,
        audio_uuid: Option<String>,
    ) -> ManiaResult<String> {
        self.download_c2c_record_inner(None, audio_uuid).await
    }

    async fn download_c2c_record_inner(
        self: &Arc<Self>,
        node: Option<IndexNode>,
        audio_uuid: Option<String>,
    ) -> ManiaResult<String> {
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
        let event: &mut RecordC2CDownloadEvent =
            downcast_mut_major_event(&mut res).ok_or(ManiaError::InternalEventDowncastError)?;
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
    ) -> ManiaResult<String> {
        let mut event = dda!(VideoC2CDownloadEvent {
            self_uid: self_uid.to_string(),
            file_name: file_name.to_string(),
            file_md5: file_md5.to_string(),
            file_sha1,
            node,
            is_group,
        });
        let res = self.send_event(&mut event).await?;
        let event: &VideoC2CDownloadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.video_url.clone())
    }

    pub async fn download_group_video(
        self: &Arc<Self>,
        group_uin: u32,
        file_name: &str,
        file_md5: &str,
        file_sha1: Option<String>,
        node: Option<IndexNode>,
    ) -> ManiaResult<String> {
        let mut event = dda!(VideoGroupDownloadEvent {
            group_uin,
            self_uid: self
                .context
                .key_store
                .uid
                .load()
                .as_ref()
                .expect("Missing self_uid")
                .as_ref()
                .clone(),
            file_name: file_name.to_string(),
            file_md5: file_md5.to_string(),
            file_sha1,
            node,
        });
        let res = self.send_event(&mut event).await?;
        let event: &VideoGroupDownloadEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.video_url.clone())
    }

    pub(crate) async fn resolve_stranger_uid2uin(
        self: &Arc<Self>,
        stranger_uid: &str,
    ) -> ManiaResult<u32> {
        let mut fetch_event = dda!(FetchUserInfoEvent {
            uid: Some(stranger_uid.to_string()),
        });
        let res = self.send_event(&mut fetch_event).await?;
        let event: &FetchUserInfoEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        Ok(event.uin)
    }

    pub(crate) async fn resolve_stranger_uid2uin_fast(self: &Arc<Self>, stranger_uid: &str) -> u32 {
        self.resolve_stranger_uid2uin(stranger_uid)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("resolve_stranger_uid2uin_fast failed: {:?}", e);
                0
            })
    }

    pub async fn fetch_group_requests(self: &Arc<Self>) -> ManiaResult<Vec<BotGroupRequest>> {
        let mut fetch_event = FetchGroupRequestsEvent::default();
        let res = self.send_event(&mut fetch_event).await?;
        let fetch_event: &FetchGroupRequestsEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;
        let mut fetch_filter_event = FetchFilteredGroupRequestsEvent::default();
        let res = self.send_event(&mut fetch_filter_event).await?;
        let fetch_filter_event: &FetchFilteredGroupRequestsEvent =
            downcast_major_event(&res).ok_or(ManiaError::InternalEventDowncastError)?;

        let all_requests: Vec<_> = fetch_event
            .results
            .iter()
            .chain(fetch_filter_event.results.iter())
            .collect();

        let requests = join_all(all_requests.into_iter().map(|req| {
            let this = Arc::clone(self);
            async move {
                let (invitor_member_uin, target_member_uin, operator_uin) = join!(
                    this.resolve_stranger_uid2uin(req.invitor_member_uid.as_deref().unwrap_or("")),
                    this.resolve_stranger_uid2uin_fast(&req.target_member_uid),
                    this.resolve_stranger_uid2uin(req.operator_uid.as_deref().unwrap_or(""))
                );
                BotGroupRequest {
                    group_uin: req.group_uin,
                    invitor_member_uin: invitor_member_uin.map(Some).unwrap_or_else(|e| {
                        tracing::error!(
                            "invitor_member_uin resolve_stranger_uid2uin error: {:?}",
                            e
                        );
                        None
                    }),
                    invitor_member_card: req.invitor_member_card.to_owned(),
                    target_member_uin,
                    target_member_card: req.target_member_card.to_owned(),
                    operator_uin: operator_uin.map(Some).unwrap_or_else(|e| {
                        tracing::error!("operator_uin resolve_stranger_uid2uin error: {:?}", e);
                        None
                    }),
                    operator_name: req.operator_name.to_owned(),
                    sequence: req.sequence,
                    state: req.state,
                    event_type: req.event_type,
                    comment: req.comment.to_owned(),
                    is_filtered: req.is_filtered,
                }
            }
        }))
        .await;

        Ok(requests)
    }
}
