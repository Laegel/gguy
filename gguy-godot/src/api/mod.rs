pub mod layer1_mutation;

use gguy_core::Document;

pub fn resolve_selector(doc: &Document, selector: &str) -> Option<usize> {
    let first = selector.chars().next()?;
    if first == '.' || first == '#' || first == '[' {
        doc.query_selector(selector)
    } else {
        doc.get_element_by_id(selector)
                .or_else(|| doc.query_selector(selector))
    }
}
