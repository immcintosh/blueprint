use std::collections::HashMap;

pub fn reassemble<'a>(
    input: impl IntoIterator<Item = impl std::borrow::Borrow<crate::markup::Blueprint>>,
) -> Vec<crate::markup::Blueprint> {
    let mut entities: HashMap<String, crate::markup::Blueprint> = HashMap::new();

    for bp in input {
        let bp = bp.borrow();
        for sec in &bp.sections {
            // The default page "?" contains every section which doesn't have an explicitly declared owner.
            let mut owner: &str = "_free_";
            // Search tags for an owner specifier.
            for tag in &sec.heading.tags {
                if tag.name.starts_with("@") {
                    owner = &tag.name[1..];
                }
            }
            if !entities.contains_key(owner) {
                entities.insert(
                    owner.to_string(),
                    crate::markup::Blueprint {
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
