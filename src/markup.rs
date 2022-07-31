use anyhow::{Context, Result};

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum TagCategory {
    #[default]
    Simple,
    Requires,
    Satisfies,
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub category: TagCategory,
    pub name: String,
}

impl Tag {
    pub fn file_name(&self) -> String {
        let prefix = match self.category {
            TagCategory::Simple => "tag_".to_string(),
            TagCategory::Requires => "req_".to_string(),
            TagCategory::Satisfies => "req_".to_string(),
        };
        prefix + &self.name + ".html"
    }
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum SpanType {
    #[default]
    Raw,
    Bold,
    Italic,
    Strikethrough,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Span {
    Plain(String),
    Bold(String),
    Italic(String),
    Strikethrough(String),
}

impl Default for Span {
    fn default() -> Self {
        Span::Plain(Default::default())
    }
}

impl Span {
    fn from_decoration(dec: char, text: String) -> Span {
        match dec {
            '*' => Span::Bold(text),
            '/' => Span::Italic(text),
            '~' => Span::Strikethrough(text),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub heading: Vec<Vec<Span>>,
    pub body: Vec<Vec<Vec<Span>>>,
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Paragraph {
    #[default]
    Empty,
    Spans(Vec<Span>),
    Table(Table),
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Heading {
    pub rank: usize,
    pub tags: Vec<Tag>,
    pub text: String,
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub heading: Heading,
    pub body: Vec<Paragraph>,
    pub subsections: Vec<Section>,
}

impl Section {
    pub fn find_tags(&self, category: TagCategory) -> Vec<&Tag> {
        self.heading
            .tags
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }
}

impl Section {
    pub fn new_root(subsections: Vec<Section>) -> Section {
        Section {
            subsections,
            ..Default::default()
        }
    }

    pub fn title(&self) -> String {
        self.heading.text.clone()
    }
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub root: Section,
}

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

    pub fn title(&self) -> Option<String> {
        Some(self.root.subsections.get(0)?.heading.text.clone())
    }
}

impl Into<crate::template::Page> for &Blueprint {
    fn into(self) -> crate::template::Page {
        crate::template::Page {
            file: ("bp_".to_string() + &self.name + ".html").into(),
            title: self.title().unwrap_or("Untitled".to_string()),
            content: self.clone(),
        }
    }
}

peg::parser! {
    grammar parse() for str {
        // Utility syntax
        rule _() -> &'input str = quiet!{$([' ' | '\t']*)}
        rule __() -> &'input str = quiet!{$(_ ['\n' | '\r']+)} / expected!("eol")
        rule ___() -> &'input str = quiet!{$(_ (![_] / __))} / expected!("eol / eof")

        // Span syntax
        rule span_decoration() -> char
            = ['*' | '/' | '~']
        rule span_decorated() -> Span
            = open:span_decoration() s:$(span_plain(<span_decoration()>)) close:span_decoration() {?
                if open == close {
                    Ok(Span::from_decoration(open, s.to_string()))
                } else {
                    Err("mismatched span delimiters")
                }
            }
        rule span_plain<T>(except: rule<T>) -> Span
            = s:$((!__ !except() [_])+) {
                Span::Plain(s.to_string())
            }
        rule span_except<T>(except: rule<T>) -> Span
            = span_decorated() / span_plain(<except()>)
        pub rule span() -> Span
            = span_decorated() / span_plain(<span_decoration()>)

        // Body syntax
        rule spans() -> Paragraph
            = __* !['#'] s:span()+ ___ { Paragraph::Spans(s) }
        rule table_row() -> Vec<Vec<Span>>
            = __* s:(span_except(<(['|'] / span_decoration())>)+) **<2,> "|" ___ {
                s
            }
        rule table_heading() -> Vec<Vec<Span>>
            = row:table_row() _ sep:((['-' | ' ']+) **<2,> "|") ___ {?
                if sep.len() == row.len() {
                    Ok(row)
                } else {
                    Err("separator count mismatch")
                }
            }
        rule table() -> Paragraph
            = head:table_heading()? rows:table_row()+ { Paragraph::Table(Table {
                heading: head.unwrap_or_default(),
                body: rows
            })}
        pub rule body() -> Vec<Paragraph>
            = (table() / spans())+

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
        rule tag() -> Tag
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
        pub rule heading(rank: usize) -> Heading
            = d:$("#"+) _ h:$(heading_words()) _ t:tags()? ___ {?
                if d.len() == rank {
                    Ok(Heading {
                        rank: d.len(),
                        tags: t.unwrap_or_default(),
                        text: h.to_string(),
                    })
                } else {
                    Err("wrong rank")
                }
            }

        // Document syntax
        pub rule section(rank: usize) -> Section
            = __* h:heading(rank) b:body()? sub:(section(rank + 1)*) {
                Section {
                    heading: h,
                    body: b.unwrap_or_default(),
                    subsections: sub,
                }
            }
        pub rule blueprint(name: &str) -> Blueprint
            = s:section(1)* ___ { Blueprint {
                name: name.to_string(),
                root: Section::new_root(s) } }
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
            root: Section::new_root(vec![Section {
                heading: Heading {
                    rank: 1,
                    tags: vec![Tag {
                        name: String::from("b"),
                        ..Default::default()
                    }],
                    text: String::from("a"),
                },
                body: vec![],
                subsections: vec![Section {
                    heading: Heading {
                        rank: 2,
                        tags: vec![],
                        text: String::from("c"),
                    },
                    body: vec![],
                    ..Default::default()
                }],
            }]),
        };
        assert_eq!(parse::blueprint(text, ""), Ok(bp));

        let text = "# a \n## b\n";
        let bp = Blueprint {
            name: String::from(""),
            root: Section::new_root(vec![Section {
                heading: Heading {
                    rank: 1,
                    tags: vec![],
                    text: String::from("a"),
                },
                body: vec![],
                subsections: vec![Section {
                    heading: Heading {
                        rank: 2,
                        tags: vec![],
                        text: String::from("b"),
                    },
                    body: vec![],
                    ..Default::default()
                }],
            }]),
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
                Paragraph::Spans(vec![Span::Plain("c".to_string())]),
                Paragraph::Spans(vec![Span::Bold("d".to_string())]),
            ],
            subsections: Default::default(),
        };
        let bp = Blueprint {
            name: String::new(),
            root: Section::new_root(vec![sec.clone()]),
        };
        assert_eq!(parse::section(text, 1), Ok(sec));
        assert_eq!(parse::blueprint(text, ""), Ok(bp));
    }

    #[test]
    fn table() {
        let text = "h|h\n-|-\na|b\nc|d";
        let par = Paragraph::Table(Table {
            heading: vec![
                vec![Span::Plain("h".to_string())],
                vec![Span::Plain("h".to_string())],
            ],
            body: vec![
                vec![
                    vec![Span::Plain("a".to_string())],
                    vec![Span::Plain("b".to_string())],
                ],
                vec![
                    vec![Span::Plain("c".to_string())],
                    vec![Span::Plain("d".to_string())],
                ],
            ],
        });
        assert_eq!(parse::body(text), Ok(vec![par]));
    }

    #[test]
    fn body() {
        let par1 = vec![Span::Plain(" a".to_string())];
        let par2 = vec![Span::Bold(" b ".to_string())];
        let text = &format!(" a\n\n* b *");
        assert_eq!(
            parse::body(text),
            Ok(vec![Paragraph::Spans(par1), Paragraph::Spans(par2)])
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
            ..Default::default()
        };
        let tagged = super::Heading {
            rank: 1,
            tags: vec![tag.clone()],
            text: String::from("a"),
        };
        let tagged_text = &format!("# {} [{}]", tagged.text, tag.name);

        assert_eq!(parse::heading(heading1_text, 1), Ok(heading1));
        assert!(parse::heading(heading2_text, 1).is_err());
        assert_eq!(parse::heading(tagged_text, 1), Ok(tagged));
    }

    #[test]
    fn span() {
        assert_eq!(parse::span("a"), Ok(Span::Plain("a".to_string())));
        assert_eq!(parse::span("*a*"), Ok(Span::Bold("a".to_string())));
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
