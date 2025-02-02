use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BotFriendGroup {
    pub group_id: u32,
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BotFriend {
    pub uin: u32,
    pub uid: String,
    pub nickname: String,
    pub remarks: String,
    pub personal_sign: String,
    pub qid: String,
    pub group: Option<BotFriendGroup>,
    pub avatar: String,
}

impl BotFriend {
    pub fn new(
        uin: u32,
        uid: String,
        nickname: String,
        remarks: String,
        personal_sign: String,
        qid: String,
        group: Option<BotFriendGroup>,
    ) -> Self {
        BotFriend {
            uin,
            uid,
            nickname,
            remarks,
            personal_sign,
            qid,
            group,
            avatar: format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", uin),
        }
    }
}
