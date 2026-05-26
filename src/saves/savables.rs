use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Footer {
    uuid: Uuid,
    header: String,
    text: String,
}
