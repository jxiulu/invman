use crate::invoice::{Denomination, Invoice, PricingMethod};

pub struct RenderSettings {
    pub font_family: String,
    pub font_size_pt: u32,
    pub margin_mm: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            font_family: "Times New Roman, Times, serif".to_string(),
            font_size_pt: 14,
            margin_mm: 12,
        }
    }
}

fn denom_symbol(denom: &Denomination) -> &'static str {
    match denom {
        Denomination::JPY => "¥",
        Denomination::USD => "$",
    }
}

/// Formats an integer with comma separators. 150000 -> "150,000".
fn fmt_amount(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes HTML and converts newlines to <br> for inline display.
fn lines(s: &str) -> String {
    escape(s).replace('\n', "<br>")
}

fn css(s: &RenderSettings) -> String {
    format!(
        r#":root {{
    --font: {font};
    --size: {size}pt;
    --margin: {margin}mm;
}}
@page {{ size: A4; margin: var(--margin); }}
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ font-family: var(--font); font-size: var(--size); }}
.invoice {{ width: 210mm; min-height: 277mm; padding: var(--margin); }}
h1 {{ font-size: 1.5em; margin-bottom: 0.5em; letter-spacing: 0.05em; }}
.meta {{ display: flex; justify-content: space-between; margin-bottom: 2em; }}
.parties {{ display: grid; grid-template-columns: 1fr 1fr; gap: 2em; margin-bottom: 2em; }}
.label {{ font-weight: bold; margin-bottom: 0.3em; }}
table {{ width: 100%; border-collapse: collapse; margin-bottom: 1.5em; }}
thead th {{ border-bottom: 1pt solid black; padding-bottom: 0.3em; text-align: left; }}
td {{ padding: 0.4em 0; }}
td.desc {{ word-break: break-word; width: 55%; }}
td.r {{ white-space: nowrap; }}
.r {{ text-align: right; }}
.total-row td {{ border-top: 1pt solid black; font-weight: bold; padding-top: 0.3em; }}
.footer {{ margin-top: 2em; }}
.footer-section {{ margin-top: 1em; }}
.footer-section .label {{ margin-bottom: 0.2em; }}
.footer-section .text {{ white-space: pre-wrap; }}"#,
        font = s.font_family,
        size = s.font_size_pt,
        margin = s.margin_mm,
    )
}

pub fn render_invoice(invoice: &Invoice, settings: &RenderSettings) -> String {
    let sym = denom_symbol(invoice.denom());
    let date = invoice.date();
    let date_str = format!("{}/{}/{}", date.year(), date.month() as u8, date.day());

    let mut item_rows = String::new();
    let mut grand_total: u32 = 0;

    for item in invoice.items() {
        let (unit_price_cell, total) = match item.price() {
            PricingMethod::UnitPrice(u) => (
                format!("{sym}{}", fmt_amount(*u)),
                item.quant() * u,
            ),
            PricingMethod::Totaled(t) => (
                "&mdash;".to_string(),
                *t,
            ),
        };
        grand_total += total;

        item_rows.push_str(&format!(
            "<tr>\
                <td class=\"desc\">{desc}</td>\
                <td class=\"r\">{quant}</td>\
                <td class=\"r\">{unit_price}</td>\
                <td class=\"r\">{sym}{total}</td>\
            </tr>\n",
            desc       = escape(item.desc()),
            quant      = item.quant(),
            unit_price = unit_price_cell,
            sym        = sym,
            total      = fmt_amount(total),
        ));
    }

    let mut footer_html = String::new();
    if !invoice.footer().is_empty() {
        footer_html.push_str("<div class=\"footer\">\n");
        for (header, text) in invoice.footer() {
            footer_html.push_str(&format!(
                "<div class=\"footer-section\">\
                    <div class=\"label\">{header}</div>\
                    <div class=\"text\">{text}</div>\
                </div>\n",
                header = escape(header),
                text   = escape(text),
            ));
        }
        footer_html.push_str("</div>\n");
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<style>
{css}
</style>
</head>
<body>
<div class="invoice">
  <h1>Invoice</h1>
  <div class="meta">
    <span>#&nbsp;{num}</span>
    <span>{date}</span>
  </div>
  <div class="parties">
    <div>
      <div class="label">To</div>
      <div>{to}</div>
    </div>
    <div>
      <div class="label">From</div>
      <div>{from}</div>
    </div>
  </div>
  <table>
    <thead>
      <tr>
        <th class="desc">Description</th>
        <th class="r">Qty</th>
        <th class="r">Unit Price</th>
        <th class="r">Total</th>
      </tr>
    </thead>
    <tbody>
{item_rows}    </tbody>
    <tfoot>
      <tr class="total-row">
        <td colspan="3">Total</td>
        <td class="r">{sym}{grand_total}</td>
      </tr>
    </tfoot>
  </table>
{footer_html}</div>
</body>
</html>"#,
        css         = css(settings),
        num         = invoice.num(),
        date        = date_str,
        to          = lines(invoice.to()),
        from        = lines(invoice.from()),
        item_rows   = item_rows,
        sym         = sym,
        grand_total = fmt_amount(grand_total),
        footer_html = footer_html,
    )
}
