use super::prelude::*;
use crate::message::chain::MessageChain;
use serde::{Deserialize, Serialize};

#[pack_content(false)]
#[derive(Default)]
pub struct MultiMsgEntity {
    pub res_id: String,
    pub chains: Vec<MessageChain>,
    pub detail_str: Option<String>,
}

/// JSON
#[derive(Serialize, Deserialize, Debug)]
pub struct MultiMsgLightApp {
    #[serde(rename = "app")]
    pub app: String,
    #[serde(rename = "config")]
    pub config: Config,
    #[serde(rename = "desc")]
    pub desc: String,
    #[serde(rename = "extra")]
    pub extra: String,
    #[serde(rename = "meta")]
    pub meta: Meta,
    #[serde(rename = "prompt")]
    pub prompt: String,
    #[serde(rename = "ver")]
    pub ver: String,
    #[serde(rename = "view")]
    pub view: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiMsgLightAppExtra {
    #[serde(rename = "filename")]
    pub filename: String,
    #[serde(rename = "tsum")]
    pub sum: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "autosize")]
    pub autosize: i64,
    #[serde(rename = "forward")]
    pub forward: i64,
    #[serde(rename = "round")]
    pub round: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "width")]
    pub width: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    #[serde(rename = "detail")]
    pub detail: Detail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detail {
    #[serde(rename = "news")]
    pub news: Vec<News>,
    #[serde(rename = "resid")]
    pub resid: String,
    #[serde(rename = "source")]
    pub source: String,
    #[serde(rename = "summary")]
    pub summary: String,
    #[serde(rename = "uniseq")]
    pub uniseq: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct News {
    #[serde(rename = "text")]
    pub text: String,
}

/// XML
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "msg")]
pub struct MultiMessage {
    #[serde(rename = "@serviceID")]
    pub service_id: Option<u32>,
    #[serde(rename = "@templateID")]
    pub template_id: Option<i32>,
    #[serde(rename = "@action")]
    pub action: Option<String>,
    #[serde(rename = "@brief")]
    pub brief: Option<String>,
    #[serde(rename = "@m_fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "@m_resid")]
    pub res_id: Option<String>,
    #[serde(rename = "@tSum")]
    pub total: Option<i32>,
    #[serde(rename = "@flag")]
    pub flag: Option<i32>,
    #[serde(rename = "item")]
    pub item: Option<MultiItem>,
    #[serde(rename = "source")]
    pub source: Option<MultiSource>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiItem {
    #[serde(rename = "@layout")]
    pub layout: Option<i32>,
    #[serde(rename = "title")]
    pub title: Option<Vec<MultiTitle>>,
    #[serde(rename = "summary")]
    pub summary: Option<MultiSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiTitle {
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@size")]
    pub size: Option<i32>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiSummary {
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiSource {
    #[serde(rename = "@name")]
    pub name: Option<String>,
}

impl Debug for MultiMsgEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "[MultiMsgEntity]: Chains: {} ResId: {}",
            self.chains.len(),
            self.res_id
        )
    }
}

impl Display for MultiMsgEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[聊天记录]")
    }
}

impl MessageEntity for MultiMsgEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        match (
            elem.rich_msg.as_ref()?.service_id,
            elem.rich_msg.as_ref()?.template1.as_ref(),
        ) {
            (Some(35), Some(template)) => {
                let xml = zlib::decompress(&template[1..])
                    .and_then(|decompressed| String::from_utf8(decompressed).ok())?;
                let xml: MultiMessage = quick_xml::de::from_str(&xml).ok()?;
                xml.res_id.map(|res_id| dda!(Self { res_id }))
            }
            _ => None,
        }
    }
}
