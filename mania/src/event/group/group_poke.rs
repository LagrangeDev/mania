pub use mania_macros::ManiaEvent;

#[derive(ManiaEvent)]
pub struct GroupPokeEvent {
    pub group_uin: u32,
    pub operator_uin: u32,
    pub target_uin: u32,
    pub action: String,
    pub suffix: String,
    pub action_img_url: String,
}
