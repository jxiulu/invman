use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, derive_more::Display)]
#[display(rename_all = "snake_case")]
pub enum SavableKind {
    Party,
    Footer
}

pub trait Savable: Serialize + for<'de> Deserialize<'de> {
    fn uuid(&self) -> Uuid;
    fn group(&self) -> SavableKind;
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
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn group(&self) -> SavableKind {
        SavableKind::Party
    }

}

#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Footer {
    uuid: Uuid,
    header: String,
    text: String,
}

impl Savable for Footer {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn group(&self) -> SavableKind {
        SavableKind::Footer
    }
}
