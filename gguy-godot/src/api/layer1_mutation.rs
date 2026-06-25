use crate::api::resolve_selector;
use crate::GguySurface;

pub fn set_text(surface: &mut GguySurface, selector: &str, value: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.set_text_by_node_id(node_id, value);
}

pub fn set_attribute(surface: &mut GguySurface, selector: &str, name: &str, value: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.set_attribute_by_node_id(node_id, name, value);
}

pub fn remove_attribute(surface: &mut GguySurface, selector: &str, name: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.remove_attribute_by_node_id(node_id, name);
}

pub fn add_class(surface: &mut GguySurface, selector: &str, class: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.add_class_by_node_id(node_id, class);
}

pub fn remove_class(surface: &mut GguySurface, selector: &str, class: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.remove_class_by_node_id(node_id, class);
}
