pub mod layer0_vdom;
pub mod layer1_mutation;

use gguy_core::Document;

pub fn resolve_selector(doc: &Document, selector: &str) -> Option<usize> {
    let first = selector.chars().next()?;
    if first == '.' || first == '#' || first == '[' || first.is_alphabetic() {
        doc.query_selector(selector)
    } else {
        doc.get_element_by_id(selector)
    }
}
