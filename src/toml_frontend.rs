use std::error::Error;

use serde::Deserialize;
use time::Date;
use crate::invoice::{Denomination, Invoice, Item};

#[derive(Deserialize)]
struct TomlItem {
    desc: String,
    #[serde(default)]
    quant: Option<String>,
    #[serde(default)]
    unit_price: Option<String>,
    #[serde(default)]
    total: Option<String>,
}

#[derive(Deserialize)]
struct TomlFooter {
    #[serde(default)]
    header: Option<String>,
    text: String,
}

#[derive(Deserialize)]
struct TomlInvoice {
    num: u32,
    to: String,
    from: String,
    denom: Denomination,

    #[serde(default)]
    date: Option<String>,

    #[serde(default)]
    ver: Option<u32>,

    #[serde(default)]
    items: Vec<TomlItem>,
    #[serde(default)]
    footer: Vec<TomlFooter>,
}

/// Strips commas and parses as u32. "150,000" -> 150000.
fn parse_num(s: &str) -> u32 {
    s.replace(',', "").trim().parse().unwrap_or(0)
}

fn parse_date(s: &str) -> Result<Date, Box<dyn Error>> {
    let parts: Vec<&str> = s.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err("date must be YY.M.D".into());
    }
    let year: i32 = parts[0].trim().parse::<i32>()? + 2000;
    let month: u8 = parts[1].trim().parse()?;
    let day: u8 = parts[2].trim().parse()?;
    Ok(Date::from_calendar_date(year, time::Month::try_from(month)?, day)?)
}

pub fn parse_invoice(toml: &str) -> Result<Invoice, Box<dyn std::error::Error>> {
    let raw: TomlInvoice = toml::from_str(toml)?;

    let date = raw.date
        .map(|s| parse_date(&s))
        .transpose()?
        .unwrap_or_else(|| {
            time::OffsetDateTime::now_utc().date()
        });

    let builder = Invoice::builder()
        .new_uuid()
        .num(raw.num)
        .date(date)
        .ver(raw.ver.unwrap_or(1))
        .denom(raw.denom)
        .to(raw.to)
        .from(raw.from);

    let items: Vec<Item> = raw.items
        .iter()
        .map(|i| {
            let quant = i.quant
                .as_deref()
                .map(|n| parse_num(n))
                .unwrap_or(1);

            let unit_price = match (i.unit_price.as_deref(), i.total.as_deref()) {
                (Some(unit_price), _) => parse_num(unit_price),
                // rate = total / quant; loses remainder if not evenly divisible
                (None, Some(t)) => parse_num(t) / quant,
                (None, None) => 0,
            };

            Item::new(i.desc.clone(), quant, unit_price)
        })
        .collect();

    let footers: Vec<(String, String)> = raw.footer
        .iter()
        .map(|f| {
            (f.header.clone().unwrap_or_else(|| "Notes".to_string()), f.text.clone())
        })
        .collect();

    Ok(builder
        .items(items)
        .footer(footers)
        .build()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(extra: &str) -> String {
        format!(
            "num = 1\nto = \"Client\"\nfrom = \"Me\"\ndenom = \"USD\"\n{extra}"
        )
    }

    // --- parse_num ---

    #[test]
    fn parse_num_plain() {
        assert_eq!(parse_num("5000"), 5000);
    }

    #[test]
    fn parse_num_commas() {
        assert_eq!(parse_num("150,000"), 150000);
    }

    #[test]
    fn parse_num_whitespace() {
        assert_eq!(parse_num("  42  "), 42);
    }

    // --- parse_date ---

    #[test]
    fn parse_date_ymd() {
        let date = parse_date("26.5.26").unwrap();
        let expected = time::Date::from_calendar_date(2026, time::Month::May, 26).unwrap();
        assert_eq!(date, expected);
    }

    #[test]
    fn parse_date_single_digit_day() {
        let date = parse_date("26.1.5").unwrap();
        let expected = time::Date::from_calendar_date(2026, time::Month::January, 5).unwrap();
        assert_eq!(date, expected);
    }

    // --- invoice fields ---

    #[test]
    fn invoice_required_fields() {
        let inv = parse_invoice(&base("")).unwrap();
        assert_eq!(*inv.num(), 1);
        assert_eq!(inv.to(), "Client");
        assert_eq!(inv.from(), "Me");
    }

    #[test]
    fn invoice_default_ver() {
        assert_eq!(*parse_invoice(&base("")).unwrap().ver(), 1);
    }

    #[test]
    fn invoice_explicit_ver() {
        assert_eq!(*parse_invoice(&base("ver = 3")).unwrap().ver(), 3);
    }

    #[test]
    fn invoice_explicit_date() {
        let inv = parse_invoice(&base("date = \"26.5.26\"")).unwrap();
        let expected = time::Date::from_calendar_date(2026, time::Month::May, 26).unwrap();
        assert_eq!(*inv.date(), expected);
    }

    #[test]
    fn invoice_default_date_is_today() {
        let inv = parse_invoice(&base("")).unwrap();
        assert_eq!(*inv.date(), time::OffsetDateTime::now_utc().date());
    }

    // --- items ---

    #[test]
    fn item_unit_price_only() {
        let inv = parse_invoice(&base("[[items]]\ndesc = \"X\"\nunit_price = \"100\"")).unwrap();
        let item = &inv.items()[0];
        assert_eq!(*item.quant(), 1);
        assert_eq!(*item.unit_price(), 100);
    }

    #[test]
    fn item_total_only_defaults_quant_to_1() {
        let inv = parse_invoice(&base("[[items]]\ndesc = \"X\"\ntotal = \"150,000\"")).unwrap();
        let item = &inv.items()[0];
        assert_eq!(*item.quant(), 1);
        assert_eq!(*item.unit_price(), 150000);
    }

    #[test]
    fn item_quant_and_unit_price() {
        let inv = parse_invoice(&base(
            "[[items]]\ndesc = \"X\"\nquant = \"16\"\nunit_price = \"5000\""
        )).unwrap();
        let item = &inv.items()[0];
        assert_eq!(*item.quant(), 16);
        assert_eq!(*item.unit_price(), 5000);
    }

    #[test]
    fn item_quant_and_total() {
        let inv = parse_invoice(&base(
            "[[items]]\ndesc = \"X\"\nquant = \"3\"\ntotal = \"15,000\""
        )).unwrap();
        let item = &inv.items()[0];
        assert_eq!(*item.quant(), 3);
        assert_eq!(*item.unit_price(), 5000);
    }

    #[test]
    fn item_unit_price_takes_priority_over_total() {
        let inv = parse_invoice(&base(
            "[[items]]\ndesc = \"X\"\nunit_price = \"200\"\ntotal = \"999\""
        )).unwrap();
        assert_eq!(*inv.items()[0].unit_price(), 200);
    }

    #[test]
    fn no_items() {
        let inv = parse_invoice(&base("")).unwrap();
        assert!(inv.items().is_empty());
    }

    // --- footer ---

    #[test]
    fn footer_default_header() {
        let inv = parse_invoice(&base("[[footer]]\ntext = \"Thank you\"")).unwrap();
        assert_eq!(inv.footer()[0].0, "Notes");
        assert_eq!(inv.footer()[0].1, "Thank you");
    }

    #[test]
    fn footer_custom_header() {
        let inv = parse_invoice(&base(
            "[[footer]]\nheader = \"Payment\"\ntext = \"Due in 30 days\""
        )).unwrap();
        assert_eq!(inv.footer()[0].0, "Payment");
        assert_eq!(inv.footer()[0].1, "Due in 30 days");
    }

    #[test]
    fn no_footer() {
        let inv = parse_invoice(&base("")).unwrap();
        assert!(inv.footer().is_empty());
    }

    // --- full example ---

    #[test]
    fn example_invoice() {
        let toml = r#"
num = 32
to = "Toei Animation"
from = "JXL"
denom = "JPY"

[[items]]
desc = "５月Restraint Fee"
total = "150,000"

[[items]]
desc = "ONP1167 KA"
quant = "16"
unit_price = "5000"

[[footer]]
text = "よろしくお願いします。"
"#;
        let inv = parse_invoice(toml).unwrap();

        assert_eq!(*inv.num(), 32);
        assert_eq!(*inv.items()[0].unit_price(), 150000);
        assert_eq!(*inv.items()[0].quant(), 1);
        assert_eq!(*inv.items()[1].quant(), 16);
        assert_eq!(*inv.items()[1].unit_price(), 5000);
        assert_eq!(inv.footer()[0].0, "Notes");
    }
}
