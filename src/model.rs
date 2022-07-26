use std::collections::HashMap;

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

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum SpanType {
    #[default]
    Raw,
    Bold,
    Italic,
    Strikethrough,
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub category: SpanType,
    pub text: String,
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Paragraph {
    pub spans: Vec<Span>,
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
}

#[derive(Clone, Default, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub sections: Vec<Section>,
}

pub fn reassemble<'a>(
    input: impl IntoIterator<Item = impl std::borrow::Borrow<Blueprint>>,
) -> Vec<Blueprint> {
    let mut entities: HashMap<String, Blueprint> = HashMap::new();

    for bp in input {
        let bp = bp.borrow();
        for sec in &bp.sections {
            // The default page "?" contains every section which doesn't have an explicitly declared owner.
            let mut owner: &str = "_free_";
            for tag in &sec.heading.tags {
                if tag.name.starts_with("@") {
                    owner = &tag.name[1..];
                }
            }
            if !entities.contains_key(owner) {
                entities.insert(
                    owner.to_string(),
                    Blueprint {
                        name: owner.to_string(),
                        ..Default::default()
                    },
                );
            }
            let page = entities.get_mut(owner).unwrap();
            // Add all the sections in this blueprint to the owner page
            page.sections.push(sec.clone());
        }
    }

    entities.into_values().collect()
}
