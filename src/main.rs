mod invoice;
mod toml_frontend;
mod codegen;
mod saving;

use std::path::Path;
use codegen::RenderSettings;

fn first_word_of(s: &str) -> String {
    s.split_whitespace()
        .next()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .unwrap_or_default()
}

fn output_stem(invoice: &invoice::Invoice) -> String {
    let from = first_word_of(invoice.from());
    let to   = first_word_of(invoice.to());

    let date = invoice.date();
    let date_str = format!(
        "{:02}{:02}{:02}",
        date.year() % 100,
        date.month() as u8,
        date.day(),
    );

    let ver = if *invoice.ver() > 1 {
        format!("_v{}", invoice.ver())
    } else {
        String::new()
    };

    format!(
        "{from}_{to}_invoice{num}{ver}_{date}",
        num  = invoice.num(),
        date = date_str,
    )
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: invman <file.toml>");
        std::process::exit(1);
    }

    let toml_path = Path::new(&args[1]);
    let out_dir   = toml_path.parent()
        .unwrap_or(Path::new("."));

    let toml_str = std::fs::read_to_string(toml_path)?;
    let invoice  = toml_frontend::parse_toml(&toml_str)?;
    let html     = codegen::render_invoice(
        &invoice,
        &RenderSettings::default()
    );

    let stem      = output_stem(&invoice);
    let html_path = out_dir.join(format!("{stem}.html"));
    let pdf_path  = out_dir.join(format!("{stem}.pdf"));

    std::fs::write(&html_path, &html)?;

    let browser = headless_chrome::Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(
        &format!("file://{}", html_path.canonicalize()?.display())
    )?;
    tab.wait_until_navigated()?;
    let pdf = tab.print_to_pdf(None)?;
    std::fs::write(&pdf_path, pdf)?;

    println!("{}", html_path.display());
    println!("{}", pdf_path.display());

    Ok(())
}
