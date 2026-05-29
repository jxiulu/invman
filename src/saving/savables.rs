use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait Savable: Serialize + for<'de> Deserialize<'de> {
}

// savable
#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Party {
    pub uuid: Uuid,
    pub name_id: String,

    /// a basic text box for name, company, address, etc.
    /// includes new lines.
    pub content: String,
}

impl Savable for Party {
}

#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Footer {
    uuid: Uuid,
    header: String,
    text: String,
}

impl Savable for Footer {
}
