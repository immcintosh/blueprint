use crate::model::*;
use anyhow::Result;

const TEMPLATE_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/template");

pub trait Page {
    fn file_name(&self) -> std::path::PathBuf;
    fn render(&self, eng: &Engine) -> Result<String>;
}

impl Page for Blueprint {
    fn file_name(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("{}.{}", self.name.replace(" ", "_"), "html"))
    }

    fn render(&self, eng: &Engine) -> Result<String> {
        let r = eng
            .tera
            .render("page.html", &tera::Context::from_serialize(&self)?)?;

        Ok(r)
    }
}

pub struct Engine {
    tera: tera::Tera,
}

impl Engine {
    pub fn new() -> Result<Engine> {
        let mut tera = tera::Tera::default();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_blueprint() -> Blueprint {
        Blueprint {
            ..Default::default()
        }
    }

    #[test]
    #[ignore]
    fn generate_page() -> Result<()> {
        std::fs::write(
            "test/template/page.html",
            make_blueprint().render(&Engine::new()?)?,
        )
        .ok();

        Ok(())
    }

    #[test]
    fn page() -> Result<()> {
        assert_eq!(
            make_blueprint().render(&Engine::new()?)?,
            include_str!("../test/template/page.html")
        );

        Ok(())
    }
}
