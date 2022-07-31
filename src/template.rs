use crate::markup::*;
use anyhow::Result;

const TEMPLATE_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/template");

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub file: std::path::PathBuf,
    pub title: String,
    pub content: Blueprint,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Context {
    pub css: Vec<String>,
    pub page: Page,
}

pub struct Engine {
    tera: tera::Tera,
}

impl Engine {
    pub fn new() -> Result<Engine> {
        let mut tera = tera::Tera::default();
        tera.register_function(
            "tag_class",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                if let Some(tag) = args.get("tag") {
                    if let Ok(tag) = tera::from_value::<Tag>(tag.clone()) {
                        match tag.category {
                            TagCategory::Simple => Ok(tera::to_value("m-label m-flat m-default")?),
                            TagCategory::Requires => Ok(tera::to_value("m-label m-warning")?),
                            TagCategory::Satisfies => Ok(tera::to_value("m-label m-success")?),
                        }
                    } else {
                        Err("'tag' is not a tag".into())
                    }
                } else {
                    Err("'tag' argument missing".into())
                }
            },
        );
        tera.register_function(
            "span_class",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                if let Some(span) = args.get("span") {
                    if let Ok(span) = tera::from_value::<Span>(span.clone()) {
                        match span.category {
                            SpanType::Raw => Ok(tera::to_value("")?),
                            SpanType::Bold => Ok(tera::to_value("m-text m-strong")?),
                            SpanType::Italic => Ok(tera::to_value("m-text m-em")?),
                            SpanType::Strikethrough => Ok(tera::to_value("m-text m-s")?),
                        }
                    } else {
                        Err("'span' is not a span".into())
                    }
                } else {
                    Err("'span' argument missing".into())
                }
            },
        );
        let templates = TEMPLATE_DIR
            .files()
            .map(|f| {
                (
                    f.path().file_name().unwrap().to_str().unwrap(),
                    f.contents_utf8().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        tera.add_raw_templates(templates)?;

        Ok(Engine { tera })
    }

    pub fn render(&self, input: &Page) -> Result<String> {
        let mut ctx = tera::Context::new();
        ctx.insert("page", input);
        ctx.insert("css", &crate::resource::THEME_DEFAULT.css_files());
        Ok(self.tera.render("page.html", &ctx)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page() -> Page {
        Page {
            title: "test".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn page() -> Result<()> {
        assert_eq!(
            Engine::new()?.render(&make_page())?,
            include_str!("../test/template/page.html")
        );

        Ok(())
    }
}
