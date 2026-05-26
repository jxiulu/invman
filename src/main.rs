mod invoice;
mod toml_frontend;
mod render;
mod saves;

use std::path::Path;
use render::RenderSettings;

fn first_line(s: &str) -> &str {
    s.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn output_stem(invoice: &invoice::Invoice) -> String {
    let from = sanitize(first_line(invoice.from()));
    let to   = sanitize(first_line(invoice.to()));
    let date = invoice.date();
    let date_str = format!("{:02}{:02}{:02}",
        date.year() % 100,
        date.month() as u8,
        date.day(),
    );

    let ver_str = if *invoice.ver() > 1 {
        format!("_v{}", invoice.ver())
    } else {
        String::new()
    };

    format!("{from}_{to}_invoice{num}{ver}_{date}",
        num  = invoice.num(),
        ver  = ver_str,
        date = date_str,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: invman <file.toml>");
        std::process::exit(1);
    }

    let toml_path = Path::new(&args[1]);
    let out_dir   = toml_path.parent().unwrap_or(Path::new("."));

    let toml_str = std::fs::read_to_string(toml_path)?;
    let invoice  = toml_frontend::parse_invoice(&toml_str)?;
    let html     = render::render_invoice(&invoice, &RenderSettings::default());

    let stem      = output_stem(&invoice);
    let html_path = out_dir.join(format!("{stem}.html"));
    let pdf_path  = out_dir.join(format!("{stem}.pdf"));

    std::fs::write(&html_path, &html)?;

    let browser = headless_chrome::Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&format!("file://{}", html_path.canonicalize()?.display()))?;
    tab.wait_until_navigated()?;
    let pdf = tab.print_to_pdf(None)?;
    std::fs::write(&pdf_path, pdf)?;

    println!("{}", html_path.display());
    println!("{}", pdf_path.display());

    Ok(())
}
