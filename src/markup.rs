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
        rule _() -> &'input str = $(" "*)

        rule end() -> &'input str = i:$("\n" / ![_])

        rule ident() -> &'input str
            = i:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']+)

        rule ident_string() -> &'input str
            = i:$(ident() ++ " ")

        pub rule tag() -> Tag
            = _ t:$("?" / ident_string()) _ {
                if t == "?" { Tag::default() }
                else { Tag { name: t.to_string() } }
            }

        rule delim_bold() -> SpanType = "*" { SpanType::Bold }

        rule delim_italic() -> SpanType = "/" { SpanType::Italic }

        rule delim_strikethrough() -> SpanType = "~" { SpanType::Strikethrough }

        rule delim() -> SpanType
            = delim_bold() / delim_italic() / delim_strikethrough()

        rule span_text_raw() -> Span
            = i:$((!delim() [^ '\n'])+) {
                Span { category: SpanType::Raw, text: i.to_string() }
            }

        rule span_text_del() -> Span
            = d1:(delim()) i:$(span_text_raw()) d2:(delim()) {?
                if d1 == d2 {
                    Ok(Span { category: d1, text: i.to_string() })
                } else {
                    Err("mismatched span delimiters")
                }
            }

        pub rule span() -> Span
            = s:(span_text_raw() / span_text_del()) { s }

        pub rule paragraph() -> Paragraph
            = !"#" s:(span()+) { Paragraph { spans: s } }

        pub rule body() -> Vec<Paragraph>
            = ['\n']* p:(paragraph() ** (['\n']+)) ['\n']* { p }

        rule heading_tags() -> Vec<Tag>
            = "[" t:(tag() ** ",") "]" { t }

        rule heading_text() -> &'input str
            = $([^ '\n' | '[' | ' ']+)

        pub rule heading() -> Heading
            = d:$("#"+) _ h:$(heading_text() ** " ") _ t:heading_tags()? end() {
                Heading {
                    rank: d.len(),
                    tags: t.unwrap_or_default(),
                    text: h.to_string()
                }
            }

        pub rule section() -> Section
            = h:heading() b:body() {
                Section { heading: h, body: b }
            }

        pub rule blueprint(name: &str) -> Blueprint
            = s:section()+ {
                Blueprint {
                    name: String::from(name),
                    sections: s
                }
            }
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
                }],
                text: String::from("a"),
            },
            body: vec![
                Paragraph {
                    spans: vec![Span {
                        category: SpanType::Raw,
                        text: String::from("c "),
                    }],
                },
                Paragraph::default(),
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
        assert_eq!(
            parse::body(text),
            Ok(vec![par1, Paragraph::default(), par2])
        );
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
        };
        let tagged = super::Heading {
            rank: 1,
            tags: vec![tag.clone()],
            text: String::from("a"),
        };
        let tagged_text = &format!("# {} [{}]", tagged.text, tag.name);

        assert_eq!(parse::heading(heading1_text), Ok(heading1));
        assert_eq!(parse::heading(heading2_text), Ok(heading2));
        assert!(!parse::heading("_a\na").is_ok());
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
        };
        let long_tag = super::Tag {
            name: String::from("this is a tag"),
        };

        assert_eq!(parse::tag("?"), Ok(Tag::default()));
        assert_eq!(parse::tag(&format!("{}", tag.name)), Ok(tag.clone()));
        assert_eq!(
            parse::tag(&format!("{}", long_tag.name)),
            Ok(long_tag.clone())
        );
    }
}
