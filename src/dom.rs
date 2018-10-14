use std::{
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
	rc::Rc,
};
use stdweb::{
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	traits::*,
	unstable::TryFrom,
	web::{document, event, HtmlElement, Node},
};

pub fn update_node(parent: &mut Node, old: &mut Option<Node>, new: &Option<Node>) {
	match (old, new) {
		(None, Some(node)) => {
			parent.append_child(node);
		},
		(Some(node), None) => {
			let _ = parent.remove_child(node).unwrap();
		},
		(Some(old_node), Some(new_node)) if !Node::is_equal_node(old_node, new_node) => {
			if !old_node.has_child_nodes() || !new_node.has_child_nodes() {
				let _ = parent.replace_child(new_node, old_node).unwrap();
			} else {
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
		_ => (),
	}
}

pub fn update_dom(state: &crate::StateRc) {
	let mut nodes = Vec::new();
	for cmp in state.borrow().mount.borrow_mut().iter_mut() {
		nodes.push(cmp.render(Rc::clone(&state)));
	}

	let crate::StateLock { style, root, styles, .. } = &mut state.borrow_mut() as &mut crate::StateLock;

	styles.borrow_mut().clear();

	style.set_text_content(&styles.borrow_mut().iter().fold(String::new(), |acc, (class, style)| {
		acc + &format!(".{} {{ {} }}", class, style)
	}));

	let root_children = root.child_nodes();

	let mut with_index: Vec<_> = nodes.into_iter().enumerate().collect();

	while let Some((index, node)) = with_index.pop() {
		update_node(root, &mut root_children.item(index as u32), &Some(node));
	}
}
