#[derive(Clone, Default, PartialEq, Debug)]
pub struct Tag {
    pub name: String,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum SpanType {
    #[default]
    Raw,
    Bold,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Span {
    pub category: SpanType,
    pub text: String,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Paragraph {
    pub spans: Vec<Span>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Heading {
    pub rank: usize,
    pub tags: Vec<Tag>,
    pub text: String,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Section {
    pub heading: Heading,
    pub body: Vec<Paragraph>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Blueprint {
    pub sections: Vec<Section>,
}

peg::parser! {
    grammar parse() for str {
        rule _() -> &'input str = $(" "*)

        rule end() -> &'input str = i:$("\n" / ![_])

        rule ident() -> &'input str
            = i:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']+)

        rule ident_string() -> &'input str
            = i:$(ident() ++ " ")

        rule tag() -> Tag
            = _ t:$("?" / ident_string()) _ {
                if t == "?" { Tag::default() }
                else { Tag { name: t.to_string() } }
            }

        pub rule tags() -> Vec<Tag>
            = t:(tag() ** ",") end() { t }

        rule delim_bold() -> SpanType = "*" { SpanType::Bold }

        rule delim() -> SpanType
            = delim_bold()

        rule span_text_raw() -> Span
            = i:$((!(delim() / (_ "#") / ("\n" _ "\n")) [_])+) {
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

        rule paragraph() -> Paragraph
            = s:(span()+) "\n"? { Paragraph { spans: s } }

        pub rule body() -> Vec<Paragraph>
            = paragraph() ** "\n"

        rule heading_tags() -> Vec<Tag>
            = "[" t:(tag() ** ",") "]" { t }

        pub rule heading() -> Heading
            = d:$("#"+) _ h:$([^ '\n' | '[' | ' '] ** " ") _ t:heading_tags()? end() {
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

        pub rule blueprint() -> Blueprint
            = s:section()+ {
                Blueprint {
                    sections: s
                }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint() {
        let text = "# a [b]\n## c";
        let bp = Blueprint {
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

        assert_eq!(parse::blueprint(text), Ok(bp));
    }

    #[test]
    fn section() {
        let text = "# a [b]\nc \n\n*d*";
        let sec = super::Section {
            heading: super::Heading {
                rank: 1,
                tags: vec![super::Tag {
                    name: String::from("b"),
                }],
                text: String::from("a"),
            },
            body: vec![
                super::Paragraph {
                    spans: vec![super::Span {
                        category: super::SpanType::Raw,
                        text: String::from("c "),
                    }],
                },
                super::Paragraph {
                    spans: vec![super::Span {
                        category: super::SpanType::Bold,
                        text: String::from("d"),
                    }],
                },
            ],
        };
        let bp = Blueprint {
            sections: vec![sec.clone()],
        };
        assert_eq!(super::parse::section(text), Ok(sec));
        assert_eq!(super::parse::blueprint(text), Ok(bp));
    }

    #[test]
    fn body() {
        let par1 = super::Paragraph {
            spans: vec![super::Span {
                category: super::SpanType::Raw,
                text: String::from(" a"),
            }],
        };
        let par2 = super::Paragraph {
            spans: vec![super::Span {
                category: super::SpanType::Bold,
                text: String::from(" b "),
            }],
        };
        let text = &format!("{}\n\n*{}*", par1.spans[0].text, par2.spans[0].text);
        assert_eq!(super::parse::body(text), Ok(vec![par1, par2]));
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

        assert_eq!(super::parse::heading(heading1_text), Ok(heading1));
        assert_eq!(super::parse::heading(heading2_text), Ok(heading2));
        assert!(!super::parse::heading("_a\na").is_ok());
        assert_eq!(super::parse::heading(tagged_text), Ok(tagged));
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

        assert_eq!(super::parse::tags("?"), Ok(vec![super::Tag::default()]));
        assert_eq!(
            super::parse::tags(&format!("{}", tag.name)),
            Ok(vec![tag.clone()])
        );
        assert_eq!(
            super::parse::tags(&format!("{}", long_tag.name)),
            Ok(vec![long_tag.clone()])
        );
        assert_eq!(
            super::parse::tags(&format!("{},{}", tag.name, tag.name)),
            Ok(vec![tag.clone(), tag.clone()])
        );
        assert_eq!(
            super::parse::tags(&format!(" {} , {} \n", tag.name, long_tag.name)),
            Ok(vec![tag.clone(), long_tag.clone()])
        );
    }
}
