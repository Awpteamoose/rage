use std::rc::Rc;
use stdweb::{
	traits::*,
	web::{document, Node},
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
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
		_ => {},
	}
}

pub fn update(state_rc: &crate::StateRc) {
	state_rc.borrow_mut().styles.borrow_mut().clear();

	let new_node = state_rc.borrow().mount.0(Rc::clone(state_rc));

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
