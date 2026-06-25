use crate::api::resolve_selector;
use crate::GguySurface;
use godot::builtin::VarDictionary;
use godot::prelude::GString;

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

pub fn create_element(surface: &mut GguySurface, parent_selector: &str, tag: &str, attrs: &VarDictionary) -> i64 {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return -1,
    };
    let parent_id = match resolve_selector(doc, parent_selector) {
        Some(id) => id,
        None => return -1,
    };
    let mut attr_pairs: Vec<(String, String)> = Vec::new();
    for key in attrs.keys_shared() {
        if let Some(val) = attrs.get(&key) {
            let key_str = key.try_to::<GString>()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let val_str = val.try_to::<GString>()
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !key_str.is_empty() {
                attr_pairs.push((key_str, val_str));
            }
        }
    }
    doc.create_element_by_node_id(parent_id, tag, &attr_pairs) as i64
}

pub fn remove_element(surface: &mut GguySurface, selector: &str) {
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let node_id = match resolve_selector(doc, selector) {
        Some(id) => id,
        None => return,
    };
    doc.remove_node_by_node_id(node_id);
}
