use std::rc::Rc;
use stdweb::{
	traits::*,
	web::{document, Node, Element},
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	unstable::{TryFrom, TryInto},
};
use std::collections::HashMap;

fn attr_updates(old: &Node, new: &Node) -> (Element, Element) {
	let old_elem = if let Ok(elem) = Element::try_from(old.clone()) { elem } else { unreachable!() };
	let new_elem = if let Ok(elem) = Element::try_from(new.clone()) { elem } else { unreachable!() };

	let old_attrs = old_elem.get_attribute_names();
	let new_attrs = new_elem.get_attribute_names();

	for attr in old_attrs.into_iter() {
		if old_elem.get_attribute(&attr).is_some() && new_elem.get_attribute(&attr).is_none() {
			old_elem.remove_attribute(&attr);
		}
	}

	for attr in new_attrs.into_iter() {
		if let (Some(new_value), Some(old_value)) = (new_elem.get_attribute(&attr), old_elem.get_attribute(&attr)) {
			if new_value != old_value {
				let _ = old_elem.set_attribute(&attr, &new_value);
			}
		}
	}

	(old_elem, new_elem)
}

pub fn update_node(parent: &mut Node, old: &mut Option<Node>, new: &Option<Node>) {
	match (old, new) {
		(None, Some(node)) => {
			parent.append_child(node);
		},
		(Some(node), None) => {
			let _ = parent.remove_child(node).unwrap();
		},
		// TODO: won't reassign event listeners
		(Some(old_node), Some(new_node)) if !Node::is_equal_node(old_node, new_node) => {
			if
				!old_node.has_child_nodes() ||
				!new_node.has_child_nodes()
			{
				let _ = parent.replace_child(new_node, old_node).unwrap();
			} else {
				let (old_element, new_element) = attr_updates(old_node, new_node);
				let _ = std::mem::replace(old_node, old_element.into());

				let old_node_children = old_node.child_nodes();
				let new_node_children = new_node.child_nodes();
				let min = u32::min(old_node_children.len(), new_node_children.len());

				for i in 0 .. min {
					update_node(old_node, &mut old_node_children.item(i), &new_node_children.item(i));
				}

				// less nodes in new than old -> remove nodes
				for i in min .. old_node_children.len() {
					let _ = parent.remove_child(&old_node_children.item(i).unwrap()).unwrap();
				}

				// less nodes in old than new -> add nodes
				for i in min .. new_node_children.len() {
					parent.append_child(&new_node_children.item(i).unwrap());
				}
			}
		},
		_ => {},
	}
}

pub fn update(state_rc: &crate::StateRc) {
	state_rc.borrow_mut().styles.borrow_mut().clear();

	let new_node = state_rc.borrow().mount.0(state_rc);

	{
		let crate::StateLock { style, styles, .. }: &mut crate::StateLock = &mut state_rc.borrow_mut();

		style.set_text_content(&styles.borrow_mut().iter().fold(String::new(), |acc, (class, style)| {
			acc + &format!(".{} {{ {} }}", class, style)
		}));

		console!(log, format!("{:?}", &styles.borrow_mut()));
	}


	let body = document().body().unwrap();
	let mut first = body.child_nodes().item(0);

	update_node(&mut Node::from(body), &mut first, &Some(new_node))
}