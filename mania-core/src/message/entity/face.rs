use super::prelude::*;
use crate::entity::sys_face::SysFaceEntry;

#[pack_content(false)]
#[derive(Default)]
pub struct FaceEntity {
    pub face_id: u16,
    pub is_large_face: bool,
    pub sys_face_entry: Option<SysFaceEntry>,
}

impl Debug for FaceEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let size = if self.is_large_face { "Large" } else { "Small" };
        write!(f, "[Face][{}]: {}", size, self.face_id)
    }
}

impl Display for FaceEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[表情]")
    }
}

impl MessageEntity for FaceEntity {
    fn pack_element(&self, _: &Context) -> Vec<Elem> {
        todo!()
    }

    fn unpack_element(elem: &Elem) -> Option<Self> {
        if let Some(face) = elem.face.as_ref()
            && face.old.is_some()
            && let Some(index) = face.index
        {
            return Some(dda!(Self {
                face_id: index as u16,
                is_large_face: false,
            }));
        }

        elem.common_elem
            .as_ref()
            .and_then(|common_elem| match common_elem.service_type {
                37 => QBigFaceExtra::decode(&*common_elem.pb_elem)
                    .ok()
                    .and_then(|qface| {
                        qface.face_id.map(|id| {
                            dda!(Self {
                                face_id: id as u16,
                                is_large_face: true,
                            })
                        })
                    }),
                33 => QSmallFaceExtra::decode(&*common_elem.pb_elem)
                    .ok()
                    .map(|small_face| {
                        dda!(Self {
                            face_id: small_face.face_id as u16,
                            is_large_face: false,
                        })
                    }),
                _ => None,
            })
    }
}
