use crate::model::*;
use anyhow::{Context, Result};

impl Blueprint {
    pub fn parse_file(file: &std::path::Path) -> Result<Blueprint> {
        Blueprint::parse(
            file.file_name()
                .context("no file")?
                .to_str()
                .context("no file")?,
            &std::fs::read_to_string(file)?,
        )
    }

    pub fn parse(name: &str, input: &str) -> Result<Blueprint> {
        Ok(parse::blueprint(input, name)?)
    }
}

peg::parser! {
    grammar parse() for str {
        // Utility syntax
        rule _() -> &'input str = quiet!{$([' ' | '\t']*)}
        rule __() -> &'input str = quiet!{$(_ ['\n' | '\r']+)} / expected!("eol")
        rule ___() -> &'input str = quiet!{$(_ (![_] / __))} / expected!("eol / eof")

        // Span syntax
        rule span_decoration() -> SpanType
            = d:['*' | '/' | '~'] {
                match d {
                    '*' => SpanType::Bold,
                    '/' => SpanType::Italic,
                    '~' => SpanType::Strikethrough,
                    _ => unreachable!()
                }
            }
        pub rule span_decorated() -> Span
            = open:span_decoration() s:$(span_plain()) close:span_decoration() {?
                if open == close {
                    Ok(Span {
                        category: open,
                        text: s.to_string()
                    })
                } else {
                    Err("mismatched span delimiters")
                }
            }
        pub rule span_plain() -> Span
            = s:$((!__ !span_decoration() [_])+) {
                Span {
                    category: SpanType::Raw,
                    text: s.to_string()
                }
            }
        pub rule span() -> Span
            = span_decorated() / span_plain()

        // Body syntax
        rule paragraph() -> Paragraph
            = __* !['#'] s:span()+ ___ { Paragraph {
                spans: s
            } }
        pub rule body() -> Vec<Paragraph>
            = paragraph()+

        // Tag syntax
        rule tag_category() -> TagCategory
            = c:(['?' | '=']?) {
                match c {
                    Some('?') => TagCategory::Requires,
                    Some('=') => TagCategory::Satisfies,
                    None => TagCategory::Simple,
                    Some(_) => unreachable!()
                }
            }
        pub rule tag() -> Tag
            = c:tag_category() t:$([^ ']' | ',']+) {
                Tag {
                    category: c,
                    name: t.to_string()
                }
            }
        pub rule tags() -> Vec<Tag>
            = ['['] t:(tag() ** ",") [']'] { t }

        // Heading syntax
        rule heading_words() -> &'input str = $(([^ '\n' | '\r' | '[' | ' ']+) ++ (" "+))
        pub rule heading() -> Heading
            = d:$("#"+) _ h:$(heading_words()) _ t:tags()? ___ {
                Heading {
                    rank: d.len(),
                    tags: t.unwrap_or_default(),
                    text: h.to_string(),
                }
            }

        // Document syntax
        pub rule section() -> Section
            = __* h:heading() b:body()? {
                Section {
                    heading: h,
                    body: b.unwrap_or_default(),
                }
            }
        pub rule blueprint(name: &str) -> Blueprint
            = s:section()* ___ { Blueprint { name: name.to_string(), sections: s }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint() -> Result<()> {
        let text = "# a [b]\n## c";
        let bp = Blueprint {
            name: String::new(),
            sections: vec![
                Section {
                    heading: Heading {
                        rank: 1,
                        tags: vec![Tag {
                            name: String::from("b"),
                            ..Default::default()
                        }],
                        text: String::from("a"),
                    },
                    body: vec![],
                },
                Section {
                    heading: Heading {
                        rank: 2,
                        tags: vec![],
                        text: String::from("c"),
                    },
                    body: vec![],
                },
            ],
        };
        assert_eq!(parse::blueprint(text, ""), Ok(bp));

        let text = "# a \n## b\n";
        let bp = Blueprint {
            name: String::from(""),
            sections: vec![
                Section {
                    heading: Heading {
                        rank: 1,
                        tags: vec![],
                        text: String::from("a"),
                    },
                    body: vec![],
                },
                Section {
                    heading: Heading {
                        rank: 2,
                        tags: vec![],
                        text: String::from("b"),
                    },
                    body: vec![],
                },
            ],
        };
        assert_eq!(parse::blueprint(text, ""), Ok(bp));

        Blueprint::parse_file(std::path::Path::new("test/sample/sample.bp"))?;

        Ok(())
    }

    #[test]
    fn section() {
        let text = "# a [b]\nc \n\n*d*";
        let sec = super::Section {
            heading: super::Heading {
                rank: 1,
                tags: vec![Tag {
                    name: String::from("b"),
                    ..Default::default()
                }],
                text: String::from("a"),
            },
            body: vec![
                Paragraph {
                    spans: vec![Span {
                        category: SpanType::Raw,
                        text: String::from("c"),
                    }],
                },
                Paragraph {
                    spans: vec![Span {
                        category: SpanType::Bold,
                        text: String::from("d"),
                    }],
                },
            ],
        };
        let bp = Blueprint {
            name: String::new(),
            sections: vec![sec.clone()],
        };
        assert_eq!(parse::section(text), Ok(sec));
        assert_eq!(parse::blueprint(text, ""), Ok(bp));
    }

    #[test]
    fn body() {
        let par1 = super::Paragraph {
            spans: vec![Span {
                category: SpanType::Raw,
                text: String::from(" a"),
            }],
        };
        let par2 = super::Paragraph {
            spans: vec![Span {
                category: SpanType::Bold,
                text: String::from(" b "),
            }],
        };
        let text = &format!("{}\n\n*{}*", par1.spans[0].text, par2.spans[0].text);
        assert_eq!(parse::body(text), Ok(vec![par1, par2]));
    }

    #[test]
    fn heading() {
        let heading1 = super::Heading {
            rank: 1,
            text: String::from("a"),
            ..Default::default()
        };
        let heading1_text = &format!("#{}", heading1.text);
        let heading2 = super::Heading {
            rank: 2,
            text: String::from("a"),
            ..Default::default()
        };
        let heading2_text = &format!("##{}", heading2.text);
        let tag = super::Tag {
            name: String::from("b"),
            ..Default::default()
        };
        let tagged = super::Heading {
            rank: 1,
            tags: vec![tag.clone()],
            text: String::from("a"),
        };
        let tagged_text = &format!("# {} [{}]", tagged.text, tag.name);

        assert_eq!(parse::heading(heading1_text), Ok(heading1));
        assert_eq!(parse::heading(heading2_text), Ok(heading2));
        assert_eq!(parse::heading(tagged_text), Ok(tagged));
    }

    #[test]
    fn span() {
        let span = Span {
            category: SpanType::Raw,
            text: String::from("a"),
        };
        let bold_span = Span {
            category: SpanType::Bold,
            text: String::from("a"),
        };
        let bold_span_text = &format!("*{}*", bold_span.text);

        assert_eq!(parse::span(&span.text), Ok(span.clone()));
        assert_eq!(parse::span(bold_span_text), Ok(bold_span.clone()));
    }

    #[test]
    fn tag() {
        let tag = super::Tag {
            name: String::from("a"),
            ..Default::default()
        };
        let long_tag = super::Tag {
            name: String::from("this is a tag"),
            ..Default::default()
        };

        assert_eq!(
            parse::tags(&format!("[{}]", tag.name)),
            Ok(vec![tag.clone()])
        );
        assert_eq!(
            parse::tags(&format!("[{}]", long_tag.name)),
            Ok(vec![long_tag.clone()])
        );
    }
}
