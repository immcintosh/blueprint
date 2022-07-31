use crate::markup::*;
use anyhow::Result;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub file: String,
    pub css: Vec<String>,
    pub title: String,
    pub content: Blueprint,
}

pub struct Requirement {
    pub name: String,
    pub content: Section,
    pub satisfies: Vec<String>,
}

pub struct Model {
    theme: &'static crate::resource::Theme,
    pages: std::collections::HashMap<String, Page>,
    requirements: std::collections::HashMap<String, Requirement>,
}

impl Model {
    pub fn new<T: IntoIterator<Item = Blueprint>>(input: T) -> Model {
        let mut model = Model {
            theme: &crate::resource::THEME_DEFAULT,
            pages: Default::default(),
            requirements: Default::default(),
        };

        let mut pages = std::collections::HashMap::new();
        for bp in input.into_iter() {
            let page = Page {
                file: "page_".to_string() + &bp.name + ".html",
                css: crate::resource::THEME_DEFAULT.css_files(),
                title: bp.name.clone(),
                content: bp.clone(),
            };
            pages.insert(bp.name.clone(), page);
        }

        for page in &pages {
            model.gather_requirements(&page.1.content.sections);
        }

        model.pages = pages;

        model
    }

    pub fn store(&self, path: &std::path::Path) -> Result<()> {
        self.theme.store(&path.join("theme"))?;
        let eng = crate::template::Engine::new()?;
        for page in &self.pages {
            std::fs::write(path.join(&page.1.file), eng.render(page.1)?)?;
        }
        for req in &self.requirements {
            std::fs::write(path.join("req_".to_string() + req.0 + ".html"), "test")?;
        }
        Ok(())
    }

    fn gather_requirements<'a, T: IntoIterator<Item = &'a Section>>(&mut self, input: T) {
        for sec in input {
            let tags = sec.find_tags(TagCategory::Requires);
            for tag in tags {
                self.requirements.insert(
                    tag.name.clone(),
                    Requirement {
                        name: tag.name.clone(),
                        content: sec.clone(),
                        satisfies: Default::default(),
                    },
                );
            }
            self.gather_requirements(&sec.subsections);
        }
    }
}
