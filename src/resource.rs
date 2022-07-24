use anyhow::{Context, Result};

pub const THEME_MCSS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/contrib/m.css/css");

pub fn store(theme: include_dir::Dir, path: &std::path::Path) -> Result<()> {
    for file in theme.files() {
        if file.path().extension().context("no extension")? == "css" {
            std::fs::write(
                path.join(file.path().file_name().context("no file")?),
                file.contents(),
            )?;
        }
    }

    Ok(())
}
