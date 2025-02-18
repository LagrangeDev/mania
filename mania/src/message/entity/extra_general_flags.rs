use super::prelude::*;
use crate::entity::bot_group_member::GroupMemberPermission;

#[pack_content(false)]
#[derive(Default, Debug)]
pub struct ExtraGeneralFlagsEntity {
    pub permission: GroupMemberPermission,
    pub new_group_level: u32,
}

impl Display for ExtraGeneralFlagsEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "") // dummy
    }
}

impl MessageEntity for ExtraGeneralFlagsEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        unreachable!("Cannot pack CommonElemExtraEntity")
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        match elem.general_flags.as_ref()?.pb_reserve {
            Some(data) => {
                let pr = data.level_permission?;
                match (pr.permission, pr.new_level) {
                    (Some(p), Some(l)) => Some(Self {
                        permission: GroupMemberPermission::try_from(p).ok().unwrap_or_default(),
                        new_group_level: l,
                    }),
                    _ => None,
                }
            }
            None => None,
        }
    }
}
