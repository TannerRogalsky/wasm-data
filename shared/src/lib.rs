use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct Person {
    pub id: u32,
    pub age: u8,
}
