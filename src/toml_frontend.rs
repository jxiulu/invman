use anyhow::Context;
use serde::Deserialize;
use time::{Date, Month};
use toml::value::Datetime;
use crate::invoice::{Denomination, Invoice, Item};

#[derive(Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum TomlNumber {
    String(String),
    U32(u32),
}

impl TryFrom<TomlNumber> for u32 {
    type Error = anyhow::Error;

    fn try_from(value: TomlNumber) -> Result<Self, Self::Error> {
        match value {
            TomlNumber::String(s) => {
                Ok(s.replace(',', "")
                    .trim()
                    .parse()?
                )
            }
            TomlNumber::U32(u) => Ok(u)
        }
    }
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum TomlDate {
    String(String),
    Datetime(Datetime)
}

impl TryFrom<TomlDate> for Date {
    type Error = anyhow::Error;

    fn try_from(value: TomlDate) -> Result<Self, Self::Error> {
        match value {
            TomlDate::String(s) => {
                let parts: Vec<&str> = s.splitn(3, '.').collect();
                if parts.len() != 3 {
                    anyhow::bail!("date must be YY.MM.DD");
                }

                let year: i32 = parts[0]
                    .trim()
                    .parse::<i32>()? + 2000;
                let month: u8 = parts[1]
                    .trim()
                    .parse()?;
                let day: u8 = parts[2]
                    .trim()
                    .parse()?;

                Ok(Date::from_calendar_date(
                    year, time::Month::try_from(month)?, day
                )?)
            }
            TomlDate::Datetime(dt) => {
                let date = dt.date.context("No date found")?;
                let month = Month::try_from(date.month)?;

                Ok(Date::from_calendar_date(
                    date.year as i32, month, date.day
                )?)
            },
        }
    }
}

#[derive(Deserialize)]
struct TomlItem {
    #[serde(alias = "name")]
    desc: String,

    #[serde(
        default = "default_quant",
        alias = "quantity", alias = "qty", alias = "q"
    )]
    quant: TomlNumber,

    #[serde(default, alias = "rate")]
    unit_price: Option<TomlNumber>,

    #[serde(default, alias = "amount")]
    total: Option<TomlNumber>,
}

fn default_quant() -> TomlNumber {
    TomlNumber::U32(1)
}

#[derive(Deserialize)]
struct TomlFooter {
    #[serde(
        default = "default_footer_header",
        alias = "title"
    )]
    header: String,

    #[serde(default, alias = "content")]
    text: String,
}

fn default_footer_header() -> String {
    "Notes".to_string()
}

#[derive(Deserialize)]
struct TomlInvoice {
    #[serde(
        default = "default_quant",
        alias = "number", alias = "no"
    )]
    num: TomlNumber,

    #[serde(alias = "recipient")]
    to: String,

    #[serde(alias = "sender")]
    from: String,

    #[serde(alias = "currency")]
    denom: Denomination,

    #[serde(default)]
    date: Option<TomlDate>,

    #[serde(
        default = "default_quant",
        alias = "version", alias = "v"
    )]
    ver: TomlNumber,

    #[serde(default, alias = "item")]
    items: Vec<TomlItem>,

    #[serde(default, alias = "footer", alias = "note", alias = "notes")]
    footer: Vec<TomlFooter>,
}

pub fn parse_toml(toml: &str) -> anyhow::Result<Invoice> {
    let toml: TomlInvoice = toml::from_str(toml)?;

    let date = toml.date
        .map(|d| d.try_into())
        .transpose()?
        .unwrap_or_else(|| {
            time::OffsetDateTime::now_utc().date()
        });

    let ver = toml.ver.try_into()?;

    let builder = Invoice::builder()
        .new_uuid()
        .num(toml.num.try_into()?)
        .date(date)
        .ver(ver)
        .denom(toml.denom)
        .to(toml.to)
        .from(toml.from);

    let mut items: Vec<Item> = Vec::new();
    for item in toml.items {
        let quant: u32 = item.quant.try_into()?;

        let item = match 
            (item.unit_price, item.total)
        {
            (None, None) => {
                Item::total(item.desc, quant, 0)
            },
            (None, Some(t)) => {
                Item::total(item.desc, quant, t.try_into()?)
            },
            (Some(u), None) => {
                Item::unit(item.desc, quant, u.try_into()?)
            }
            (Some(u), Some(t)) => {
                anyhow::bail!("Please either unit price your item or total it.")
            }
        };

        items.push(item);
    }

    let footers: Vec<(String, String)> = toml.footer
        .into_iter()
        .map(|f| {
            (f.header, f.text)
        })
        .collect();

    Ok(builder
        .items(items)
        .footer(footers)
        .build()
    )
}
