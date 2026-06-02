use getset::Getters;
use serde::{
    Deserialize, Serialize, de::DeserializeOwned
};
use uuid::Uuid;

pub trait Savable: Serialize + for<'de> Deserialize<'de> {
}

impl<T> Savable for T
where 
    T: Serialize + DeserializeOwned
{}

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
