use std::time::Instant;

use crate::GguySurface;
use gguy_core::NodeDescriptor;
use godot::builtin::{VarArray, VarDictionary};
use godot::prelude::godot_print;

pub fn render(surface: &mut GguySurface, tree: VarDictionary) {
    let _t = Instant::now();
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => {
            godot_print!("render: no document (call load_html first)");
            return;
        }
    };
    let descriptors = dict_to_nodes(&tree);
    let dict_us = _t.elapsed().as_micros();
    let _t = Instant::now();
    doc.reconcile(None, &descriptors);
    let reconcile_us = _t.elapsed().as_micros();
    godot_print!("[profile] render: dict_to_nodes={}µs  reconcile={}µs", dict_us, reconcile_us);
}

pub fn render_into(surface: &mut GguySurface, container_id: &str, tree: VarDictionary) {
    let _t = Instant::now();
    let doc = match &mut surface.document {
        Some(doc) => doc,
        None => return,
    };
    let descriptors = dict_to_nodes(&tree);
    let dict_us = _t.elapsed().as_micros();
    let _t = Instant::now();
    doc.reconcile(Some(container_id), &descriptors);
    let reconcile_us = _t.elapsed().as_micros();
    godot_print!("[profile] render_into: dict_to_nodes={}µs  reconcile={}µs", dict_us, reconcile_us);
}

fn dict_to_nodes(dict: &VarDictionary) -> Vec<NodeDescriptor> {
    let mut nodes = Vec::new();
    // Check if it's an array-like dict (has numeric keys) or a single node.
    // We just build one node from the dict.
    nodes.push(dict_to_node(dict));
    nodes
}

fn dict_to_node(dict: &VarDictionary) -> NodeDescriptor {
    let _t = Instant::now();

    let tag = dict
        .get("tag")
        .and_then(|v| v.try_to::<godot::builtin::GString>().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let text = dict
        .get("text")
        .and_then(|v| v.try_to::<godot::builtin::GString>().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let attrs = parse_attrs(dict);
    let children = parse_children(dict);

    let elapsed = _t.elapsed();
    if elapsed.as_micros() > 50 {
        godot_print!("[profile] dict_to_node: tag={:?} took {}µs  ({} attrs, {} children)",
            tag, elapsed.as_micros(), attrs.len(), children.len());
    }

    NodeDescriptor {
        tag,
        attrs,
        children,
        text,
    }
}

fn parse_attrs(dict: &VarDictionary) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    if let Some(attrs_var) = dict.get("attrs") {
        if let Ok(attrs_dict) = attrs_var.try_to::<VarDictionary>() {
            for key in attrs_dict.keys_shared() {
                if let Some(val) = attrs_dict.get(&key) {
                    let key_str = key.try_to::<godot::builtin::GString>()
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let val_str = val.try_to::<godot::builtin::GString>()
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    if !key_str.is_empty() {
                        attrs.push((key_str, val_str));
                    }
                }
            }
        }
    }
    attrs
}

fn parse_children(dict: &VarDictionary) -> Vec<NodeDescriptor> {
    let mut children = Vec::new();
    if let Some(children_var) = dict.get("children") {
        if let Ok(children_arr) = children_var.try_to::<VarArray>() {
            for item in children_arr.iter_shared() {
                if let Ok(child_dict) = item.try_to::<VarDictionary>() {
                    children.push(dict_to_node(&child_dict));
                }
            }
        }
    }
    children
}
