fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=RECURSIVE_FONT_DIR");
    println!("cargo:rerun-if-env-changed=LEPTOS_SITE_ROOT");

    let font_source = format!("{}/Recursive_VF.woff2", env!("RECURSIVE_FONT_DIR"));
    let font_source = std::path::Path::new(font_source.as_str());
    let font_source = std::fs::canonicalize(font_source)?;

    let target_dir = std::env::var("LEPTOS_SITE_ROOT").unwrap_or("target/site".to_string());
    println!("{target_dir}");
    let font_out = format!("{target_dir}/site-font.woff2");
    let font_out = std::path::Path::new(font_out.as_str());

    std::fs::copy(font_source, font_out)?;
    Ok(())
}
