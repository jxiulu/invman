use uuid::Uuid;
use getset::Getters;
use serde::{Serialize, Deserialize};
use time::Date;

#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Item {
    desc: String,
    quant: u32,

    /// currency-agnostic. invoice determines the currency.
    /// only the rate is held: setting unit price multiplies quantity by unit price.
    rate: u32,
}

/// Immutable image of an invoice
#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Invoice {
    uuid: Uuid,
    num: u32,
    ver: u32,
    date: Date,

    denom: Denomination,
    
    to: String,
    from: String,

    items: Vec<Item>,

    footer: Vec<(String, String)>
}

#[derive(Deserialize, Serialize)]
pub enum Denomination {
    JPY,
    USD
}

pub struct InvoiceBuilder {
    uuid: Option<Uuid>,
    num: Option<u32>,
    date: Option<Date>,
    ver: Option<u32>,

    denom: Option<Denomination>,
    
    to: Option<String>,
    from: Option<String>,

    items: Vec<Item>,

    footer: Vec<(String, String)>
}

impl InvoiceBuilder {
    pub fn new() -> Self {
        Self {
            uuid: None,
            num: None,
            date: None,
            ver: None,
            denom: None,
            to: None,
            from: None,
            items: Vec::new(),
            footer: Vec::new()
        }
    }

    pub fn new_uuid(mut self) -> Self {
        self.uuid = Some(Uuid::now_v7());
        self
    }

    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn num(mut self, num: u32) -> Self {
        self.num = Some(num);
        self
    }

    pub fn date(mut self, date: Date) -> Self {
        self.date = Some(date);
        self
    }

    pub fn ver(mut self, ver: u32) -> Self {
        self.ver = Some(ver);
        self
    }

    pub fn denom(mut self, denom: Denomination) -> Self {
        self.denom = Some(denom);
        self
    }

    pub fn to(mut self, to: String) -> Self {
        self.to = Some(to);
        self
    }

    pub fn from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }

    pub fn unit_item(mut self, desc: String, quant: u32, rate: u32) -> Self {
        self.items.push(Item {
            desc,
            quant,
            rate,
        });
        self
    }

    pub fn footer(mut self, header: String, text: String) -> Self {
        self.footer.push((header, text));
        self
    }

    pub fn build(mut self) -> Invoice {
        Invoice {
            uuid: self.uuid.expect("Missing uuid"),
            num: self.num.expect("Missing num"),
            date: self.date.expect("Missing date"),
            ver: self.ver.expect("Missing ver"),
            denom: self.denom.expect("Missing denom"),
            to: self.to.expect("Missing to"),
            from: self.from.expect("Missing from"),
            items: self.items,
            footer: self.footer,
        }
    }
}
