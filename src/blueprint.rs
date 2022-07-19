use pest::iterators::{Pair, Pairs};

#[derive(Clone, Default)]
pub struct Tag {
    pub(crate) name: String,
}

impl Tag {
    pub fn parse(pair: Pair<Rule>) -> Self {
        Self {
            name: String::from(pair.as_str()),
        }
    }
}

#[derive(Clone, Default)]
pub struct Content {
    pub(crate) data: String,
}

impl Content {
    pub fn parse(pairs: Pairs<Rule>) -> Self {
        Self {
            data: pairs.fold(String::new(), |a, x| match x.as_rule() {
                Rule::body_element => a + x.as_str(),
                _ => unreachable!(),
            }),
        }
    }
}

#[derive(Clone, Default)]
pub struct Block {
    pub(crate) tags: Vec<Tag>,
    pub(crate) content: Content,
}

impl Block {
    fn parse(pairs: Pairs<Rule>) -> Self {
        let tags = pairs.clone().filter_map(|x| match x.as_rule() {
            Rule::tag => Some(Tag::parse(x)),
            _ => None,
        });
        let content = pairs.clone().find_map(|x| match x.as_rule() {
            Rule::body => Some(Content::parse(x.into_inner())),
            _ => None,
        });
        Self {
            tags: tags.collect(),
            content: content.unwrap_or_default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Blueprint {
    pub(crate) blocks: Vec<Block>,
}

impl Blueprint {
    pub fn parse(data: &str) -> Result<Blueprint, Box<dyn std::error::Error>> {
        use pest::Parser;

        let result = BlueprintParser::parse(Rule::blueprint, data);
        if let Err(e) = result {
            Err(Box::new(e))
        } else {
            Ok(Blueprint {
                blocks: result
                    .unwrap()
                    .filter_map(|x| match x.as_rule() {
                        Rule::block => Some(Block::parse(x.into_inner())),
                        Rule::EOI => None,
                        _ => unreachable!(),
                    })
                    .collect(),
            })
        }
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar/blueprint.pest"]
struct BlueprintParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::consumes_to;
    use pest::parses_to;

    #[test]
    fn test_parse() {
        let blueprint = super::Blueprint::parse(include_str!("../sample/sample.bp"));
        assert!(blueprint.is_ok());
        let blueprint = blueprint.unwrap();
        assert_eq!(blueprint.blocks.len(), 2);
        let block = &blueprint.blocks[0];
        assert_eq!(block.tags.len(), 1);
        assert_eq!(block.tags[0].name, "parsing");
        assert_eq!(block.content.data, "_section_\n__subsection__\n\n");
        let block = &blueprint.blocks[1];
        assert_eq!(block.tags.len(), 2);
        assert_eq!(block.tags[0].name, "test");
        assert_eq!(block.tags[1].name, "tag");
        assert_eq!(block.content.data, "asdf\n");
    }

    #[test]
    fn test_tags() {
        parses_to! {
            parser: BlueprintParser,
            input: "",
            rule: Rule::sol,
            tokens: []
        }

        parses_to! {
            parser: BlueprintParser,
            input: "\n",
            rule: Rule::sol,
            tokens: []
        }

        parses_to! {
            parser: BlueprintParser,
            input: "@[a]",
            rule: Rule::tag_group,
            tokens: [
                tag(2, 3)
            ]
        }

        parses_to! {
            parser: BlueprintParser,
            input: "@[a,b]",
            rule: Rule::tag_group,
            tokens: [
                tag(2, 3),
                tag(4, 5)
            ]
        }

        parses_to! {
            parser: BlueprintParser,
            input: "foo `bar`foo*bar*",
            rule: Rule::text,
            tokens: [
                text(0, 17, [
                    span(0, 3),
                    span(4, 9),
                    span(9, 12),
                    span(12, 17)
                ])
            ]
        }

        parses_to! {
            parser: BlueprintParser,
            input: "_a_\n__b__\nnow is the time for all good men to come to the aid of their\ncountry",
            rule: Rule::body,
            tokens: [
                body(0, 78, [
                    body_element_sec(0, 3),
                    body_element_sec(4, 9),
                    body_element_par(10, 70),
                    body_element_par(71, 78)
                ])
            ]
        }
    }
}
