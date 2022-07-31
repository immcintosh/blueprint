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
            "tag_link",
            |args: &std::collections::HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                if let Some(tag) = args.get("tag") {
                    if let Ok(tag) = tera::from_value::<Tag>(tag.clone()) {
                        Ok(tera::to_value(tag.file_name())?)
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
                    match tera::from_value::<Span>(span.clone()) {
                        Ok(Span::Plain(_)) => Ok(tera::to_value("")?),
                        Ok(Span::Bold(_)) => Ok(tera::to_value("m-text m-strong")?),
                        Ok(Span::Italic(_)) => Ok(tera::to_value("m-text m-em")?),
                        Ok(Span::Strikethrough(_)) => Ok(tera::to_value("m-text m-s")?),
                        Err(_) => Err("'span' is not a span".into()),
                    }
                } else {
                    Err("'span' argument missing".into())
                }
            },
        );
        tera.register_filter(
            "span_text",
            |val: &tera::Value,
             _args: &std::collections::HashMap<String, tera::Value>|
             -> tera::Result<tera::Value> {
                match tera::from_value::<Span>(val.clone()) {
                    Ok(Span::Plain(s)) => Ok(tera::to_value(s)?),
                    Ok(Span::Bold(s)) => Ok(tera::to_value(s)?),
                    Ok(Span::Italic(s)) => Ok(tera::to_value(s)?),
                    Ok(Span::Strikethrough(s)) => Ok(tera::to_value(s)?),
                    Err(_) => Err("not a span".into()),
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
