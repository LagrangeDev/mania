use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SysFaceEntry {
    pub q_sid: String,
    pub q_des: Option<String>,
    pub em_code: Option<String>,
    pub q_cid: Option<i32>,
    pub ani_sticker_type: Option<i32>,
    pub ani_sticker_pack_id: Option<i32>,
    pub ani_sticker_id: Option<i32>,
    pub url: Option<String>,
    pub emoji_name_alias: Option<Vec<String>>,
    pub ani_sticker_width: Option<i32>,
    pub ani_sticker_height: Option<i32>,
}

impl SysFaceEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        q_sid: String,
        q_des: Option<String>,
        em_code: Option<String>,
        q_cid: Option<i32>,
        ani_sticker_type: Option<i32>,
        ani_sticker_pack_id: Option<i32>,
        ani_sticker_id: Option<i32>,
        url: Option<String>,
        emoji_name_alias: Option<Vec<String>>,
        ani_sticker_width: Option<i32>,
        ani_sticker_height: Option<i32>,
    ) -> Self {
        SysFaceEntry {
            q_sid,
            q_des,
            em_code,
            q_cid,
            ani_sticker_type,
            ani_sticker_pack_id,
            ani_sticker_id,
            url,
            emoji_name_alias,
            ani_sticker_width,
            ani_sticker_height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SysFacePackEntry {
    pub emoji_pack_name: String,
    pub emojis: Vec<SysFaceEntry>,
}

impl SysFacePackEntry {
    pub fn new(emoji_pack_name: String, emojis: Vec<SysFaceEntry>) -> Self {
        SysFacePackEntry {
            emoji_pack_name,
            emojis,
        }
    }

    pub fn get_unique_super_qsids(
        &self,
        exclude_ani_sticker_types_and_pack_ids: &[(i32, i32)],
    ) -> Vec<u32> {
        self.emojis
            .iter()
            .filter(|e| {
                e.ani_sticker_type.is_some()
                    && e.ani_sticker_pack_id.is_some()
                    && !exclude_ani_sticker_types_and_pack_ids
                        .contains(&(e.ani_sticker_type.unwrap(), e.ani_sticker_pack_id.unwrap()))
            })
            .map(|e| e.q_sid.parse::<u32>().unwrap())
            .collect()
    }
}
