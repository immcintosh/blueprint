use crate::markup::*;
use anyhow::Result;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Requirement {
    pub name: String,
    pub content: Section,
    pub satisfies: Vec<Tag>,
}

impl Into<crate::template::Page> for &Requirement {
    fn into(self) -> crate::template::Page {
        crate::template::Page {
            file: ("req_".to_string() + &self.name + ".html").into(),
            title: self.content.title(),
            content: Blueprint {
                name: self.name.clone(),
                root: Section::new_root(vec![self.content.clone()]),
            },
        }
    }
}

pub struct Model {
    theme: &'static crate::resource::Theme,
    pages: std::collections::HashMap<String, Blueprint>,
    requirements: std::collections::HashMap<String, Requirement>,
}

impl Model {
    pub fn new<T: IntoIterator<Item = Blueprint>>(input: T) -> Model {
        let pages: std::collections::HashMap<String, Blueprint> =
            input.into_iter().map(|bp| (bp.name.clone(), bp)).collect();
        Model {
            theme: &crate::resource::THEME_DEFAULT,
            requirements: Model::requirements(&pages, None),
            pages: pages,
        }
    }

    pub fn store(&self, path: &std::path::Path) -> Result<()> {
        self.theme.store(&path.join("theme"))?;
        let eng = crate::template::Engine::new()?;
        for bp in &self.pages {
            let page: crate::template::Page = bp.1.into();
            std::fs::write(path.join(&page.file), eng.render(&page)?)?;
        }
        for req in &self.requirements {
            let page: crate::template::Page = req.1.into();
            std::fs::write(path.join(&page.file), eng.render(&page)?)?;
        }
        Ok(())
    }

    fn requirements<'a>(
        pages: &'a std::collections::HashMap<String, Blueprint>,
        input: Option<&'a Section>,
    ) -> std::collections::HashMap<String, Requirement> {
        let mut out: std::collections::HashMap<String, Requirement> =
            std::collections::HashMap::new();
        if let Some(sec) = input {
            for tag in sec.find_tags(TagCategory::Requires) {
                if let Some(old) = out.insert(
                    tag.name.clone(),
                    Requirement {
                        name: tag.name.clone(),
                        content: sec.clone(),
                        satisfies: sec
                            .find_tags(TagCategory::Satisfies)
                            .iter()
                            .cloned()
                            .cloned()
                            .collect(),
                    },
                ) {
                    eprintln!("Duplicate requirement: {}", old.name);
                }
            }
            for sec in &sec.subsections {
                out.extend(Self::requirements(pages, Some(sec)))
            }
        } else {
            for page in pages {
                for sec in &page.1.root.subsections {
                    out.extend(Self::requirements(pages, Some(sec)));
                }
            }
        }
        out
    }
}
