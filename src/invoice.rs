use uuid::Uuid;
use getset::Getters;
use serde::{Serialize, Deserialize};
use time::Date;
use bon::Builder;

#[derive(Serialize, Deserialize)]
pub enum PricingMethod {
    Totaled(u32),
    UnitPrice(u32),
}

#[derive(Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Item {
    desc: String,
    quant: u32,
    price: PricingMethod,
}

impl Item {
    pub fn unit(desc: String, quant: u32, unit_price: u32) -> Self {
        Self {
            desc,
            quant,
            price: PricingMethod::UnitPrice(unit_price)
        }
    }

    pub fn total(desc: String, quant: u32, total: u32) -> Self {
        Self {
            desc,
            quant,
            price: PricingMethod::Totaled(total)
        }
    }
}

/// Immutable image of an invoice
#[derive(Getters, Serialize, Deserialize, Builder)]
#[getset(get = "pub")]
pub struct Invoice {
    uuid: Uuid,
    num: u32,
    ver: u32,
    date: Date,

    denom: Denomination,
    
    to: String,
    from: String,
    
    #[builder(default)]
    items: Vec<Item>,

    #[builder(default)]
    footer: Vec<(String, String)>
}

use invoice_builder::{State, IsUnset, SetUuid};
impl<S: State> InvoiceBuilder<S> {
    pub fn new_uuid(self) -> InvoiceBuilder<SetUuid<S>> 
    where 
        S::Uuid: IsUnset
    {
        self.uuid(Uuid::now_v7())
    }
}

#[derive(Deserialize, Serialize)]
pub enum Denomination {
    JPY,
    USD
}
