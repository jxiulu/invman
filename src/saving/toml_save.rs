use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};

use serde::{Deserialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{
    invoice::*,
    saving::savables::*,
};

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

const FOOTERS_FOLDER: &str = "footers";
const PARTIES_FOLDER: &str = "parties";
const INVOICES_FOLDER: &str = "invoices";

/// owns anything in invoice.rs that is savable
pub struct Storage {
    footers: HashMap<Uuid, Footer>,
    parties: HashMap<Uuid, Party>,
    invoices: HashMap<Uuid, Invoice>,

    /// top-level db folder.
    db: PathBuf,
}

pub trait Stores<T: Savable> {
    fn save(&mut self, item: T) -> anyhow::Result<()>;
    fn load(&self, uuid: Uuid) -> Option<&T>;
}

impl Stores<Footer> for Storage {
    fn save(&mut self, item: Footer) -> anyhow::Result<()> {
        let uuid = *item.uuid();
        self.save_file(&item, FOOTERS_FOLDER, uuid)?;
        self.footers.insert(uuid, item);

        Ok(())
    }

    fn load(&self, uuid: Uuid) -> Option<&Footer> {
        self.footers.get(&uuid)
    }
}

impl Stores<Party> for Storage {
    fn save(&mut self, item: Party) -> anyhow::Result<()> {
        let uuid = *item.uuid();
        self.save_file(&item, PARTIES_FOLDER, uuid)?;
        self.parties.insert(uuid, item);

        Ok(())
    }

    fn load(&self, uuid: Uuid) -> Option<&Party> {
        self.parties.get(&uuid)
    }
}

impl Stores<Invoice> for Storage {
    fn save(&mut self, item: Invoice) -> anyhow::Result<()> {
        let uuid = *item.uuid();
        self.save_file(&item, INVOICES_FOLDER, uuid)?;
        self.invoices.insert(uuid, item);

        Ok(())
    }

    fn load(&self, uuid: Uuid) -> Option<&Invoice> {
        self.invoices.get(&uuid)
    }
}

impl Storage {
    fn save_file<T: Savable>(
        &self, item: &T, folder: &str, uuid: Uuid
    ) -> anyhow::Result<()> {
        let path = self.db
            .join(folder)
            .join(format!("{uuid}.toml"));

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, toml::to_string(&item)?)?;

        Ok(())
    }

    pub fn new(db: PathBuf) -> Self {
        Self {
            footers: load_dir(&db.join(FOOTERS_FOLDER)),
            parties: load_dir(&db.join(PARTIES_FOLDER)),
            invoices: load_dir(&db.join(INVOICES_FOLDER)),
            db,
        }
    }
}
