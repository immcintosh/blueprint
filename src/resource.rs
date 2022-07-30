use anyhow::Result;

trait ExtractIf {
    fn extract_if<S: AsRef<std::path::Path>, F>(
        &self,
        base_path: S,
        pred: F,
    ) -> std::io::Result<()>
    where
        F: FnMut(&include_dir::DirEntry) -> bool;
}

impl<'a> ExtractIf for include_dir::Dir<'a> {
    fn extract_if<S: AsRef<std::path::Path>, F>(
        &self,
        base_path: S,
        mut pred: F,
    ) -> std::io::Result<()>
    where
        F: FnMut(&'a include_dir::DirEntry) -> bool,
    {
        let base_path = base_path.as_ref();

        for entry in self.entries() {
            let path = base_path.join(entry.path());

            if pred(entry) {
                match entry {
                    include_dir::DirEntry::Dir(d) => {
                        std::fs::create_dir_all(&path)?;
                        d.extract(base_path)?;
                    }
                    include_dir::DirEntry::File(f) => {
                        if let Some(p) = f.path().parent() {
                            std::fs::create_dir_all(p)?;
                        }
                        std::fs::write(path, f.contents())?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct Theme<'a> {
    dir: &'a include_dir::Dir<'a>,
    use_css: &'a [&'a str],
}

impl<'a> Theme<'a> {
    pub fn extract(&self, path: &std::path::Path) -> Result<()> {
        self.dir.extract_if(path, |entry| {
            entry.path().extension() == Some(std::ffi::OsStr::new("css"))
        })?;

        Ok(())
    }
}

const THEME_MCSS: Theme = Theme {
    dir: &include_dir::include_dir!("$CARGO_MANIFEST_DIR/contrib/m.css/css"),
    use_css: &[],
};

pub const THEME_MCSS_DARK: Theme = Theme {
    use_css: &["m-dark.css"],
    ..THEME_MCSS
};

pub const THEME_MCSS_LIGHT: Theme = Theme {
    use_css: &["m-light.css"],
    ..THEME_MCSS
};

pub const THEME_DEFAULT: Theme = THEME_MCSS_DARK;
