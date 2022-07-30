use crate::markup::*;
use anyhow::Result;

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub file: String,
    pub css: Vec<String>,
    pub title: String,
    pub content: Blueprint,
}

pub struct Model {
    theme: &'static crate::resource::Theme,
    pages: Vec<Page>,
}

impl Model {
    pub fn new<T: IntoIterator<Item = Blueprint>>(input: T) -> Self {
        let theme = &crate::resource::THEME_DEFAULT;
        Model {
            theme: theme,
            pages: input
                .into_iter()
                .map(|bp| Page {
                    file: "page_".to_string() + &bp.name + ".html",
                    css: theme.css_files(),
                    title: bp.name.clone(),
                    content: bp.clone(),
                })
                .collect(),
        }
    }

    pub fn store(&self, path: &std::path::Path) -> Result<()> {
        self.theme.extract(&path.join("theme"))?;
        let eng = crate::template::Engine::new()?;
        for page in &self.pages {
            std::fs::write(path.join(&page.file), eng.render(page)?)?;
        }
        Ok(())
    }
}
