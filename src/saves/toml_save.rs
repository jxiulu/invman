use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::PathBuf,
};
use serde::de::DeserializeOwned;
use derive_more::Display;
use crate::{
    invoice::*,
    saves::{self, savables::*},
};
use uuid::Uuid;

fn load_dir<T: DeserializeOwned>(dir: &PathBuf) -> HashMap<Uuid, T> {
    let mut map = HashMap::new();

    let Ok(entries) = fs::read_dir(dir)
    else {
        return map;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let ext = path
            .extension()
            .and_then(|e| e.to_str());
        if ext != Some("toml") {
            continue;
        }

        let Some(Ok(uuid)) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.parse::<Uuid>())
        else {
            continue;
        };

        let Ok(content) = fs::read_to_string(&path)
        else { 
            continue; 
        };
        let Ok(value) = toml::from_str(&content) 
        else { 
            continue; 
        };

        map.insert(uuid, value);
    }

    map
}

/// owns anything in invoice.rs that is savable
pub struct Storage {
    footers: HashMap<Uuid, Footer>,
    parties: HashMap<Uuid, Party>,
    invoices: HashMap<Uuid, Invoice>,

    /// top-level db folder.
    db: PathBuf,
}

impl Storage {

    pub fn new(db: PathBuf) -> Self {
        Self {
            footers: load_dir(&db.join("fields")),
            parties: load_dir(&db.join("parties")),
            invoices: load_dir(&db.join("invoices")),
            db,
        }
    }

    pub fn footer(&self, uuid: &Uuid) -> Option<&Footer> {
        self.footers.get(uuid)
    }

    pub fn party(&self, uuid: &Uuid) -> Option<&Party> {
        self.parties.get(uuid)
    }

    pub fn invoice(&self, uuid: &Uuid) -> Option<&Invoice> {
        self.invoices.get(uuid)
    }

    pub fn save_footer(
        &mut self, field: Footer
    ) -> Result<(), anyhow::Error> {
        let uuid = *field.uuid();
        let path = self.db
            .join("fields")
            .join(format!("{uuid}.toml"));

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, toml::to_string(&field)?)?;

        self.footers.insert(uuid, field);

        Ok(())
    }

    pub fn save_party(
        &mut self, party: Party
    ) -> Result<(), anyhow::Error> {
        let uuid = *party.uuid();
        let path = self.db
            .join("parties")
            .join(format!("{uuid}.toml"));

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, toml::to_string(&party)?)?;

        self.parties.insert(uuid, party);

        Ok(())
    }

    pub fn save_invoice(
        &mut self, invoice: Invoice
    ) -> Result<(), anyhow::Error> {

        let uuid = *invoice.uuid();
        let path = self.db
            .join("invoices")
            .join(format!("{uuid}.toml"));

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, toml::to_string(&invoice)?)?;

        self.invoices.insert(uuid, invoice);

        Ok(())
    }

}
