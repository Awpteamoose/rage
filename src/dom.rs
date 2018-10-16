use stdweb::{
	traits::*,
	unstable::TryFrom,
	web::{document, Element, Node},
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
};

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn update_attributes(old: &Element, new: &Element) {
	let old_attrs = old.get_attribute_names();
	let new_attrs = new.get_attribute_names();

	for attr in old_attrs.into_iter() {
		if old.get_attribute(&attr).is_some() && new.get_attribute(&attr).is_none() {
			old.remove_attribute(&attr);
		}
	}

	for attr in new_attrs.into_iter() {
		if let (Some(new_value), Some(old_value)) = (new.get_attribute(&attr), old.get_attribute(&attr)) {
			if new_value != old_value {
				let _ = old.set_attribute(&attr, &new_value);
			}
		}
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update_node(parent: &mut Element, old: Option<Node>, new: Option<Node>) {
	match (old, new) {
		(None, Some(new_node)) => {
			console!(log, "append on compare");
			parent.append_child(&new_node);
		},
		(Some(old_node), None) => {
			console!(log, "remove on compare");
			let _ = parent.remove_child(&old_node);
		},
		(Some(old_node), Some(new_node)) => {
			if Node::is_equal_node(&old_node, &new_node) {
				return;
			}

			if !old_node.has_child_nodes() || !new_node.has_child_nodes() {
				let _ = parent.replace_child(&new_node, &old_node);
				return;
			}

			let (mut old_elem, new_elem) = (Element::try_from(old_node).unwrap(), Element::try_from(new_node).unwrap());
			update_attributes(&old_elem, &new_elem);

			let old_elem_children = old_elem.child_nodes();
			let new_elem_children = new_elem.child_nodes();

			let old_elem_children_len = old_elem_children.len();
			let new_elem_children_len = new_elem_children.len();

			let min = u32::min(old_elem_children_len, new_elem_children_len);

			let mut to_update = Vec::new();
			let mut to_remove = Vec::new();
			let mut to_append = Vec::new();

			for i in 0..min {
				to_update.push((old_elem_children.item(i), new_elem_children.item(i)));
			}
			for i in min..old_elem_children_len {
				console!(log, "REMOVE");
				to_remove.push(old_elem_children.item(i).unwrap());
			}
			for i in min..new_elem_children_len  {
				console!(log, "ADD");
				to_append.push(new_elem_children.item(i).unwrap());
			}

			for (a, b) in to_update.into_iter() {
				update_node(&mut old_elem, a, b);
			}
			for node in to_remove.into_iter() {
				let _ = parent.remove_child(&node);
			}
			for node in to_append.into_iter() {
				parent.append_child(&node);
			}
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default + 'static>(state_lock: &'static crate::cmp::StateLock<'static, S>) {
	console!(log, "RERENDER");
	state_lock.view_meta().styles.write().unwrap().clear();

	let new_node = (state_lock.view_meta().mount)();

	{
		let crate::cmp::StateMeta { styles, style, .. }: &mut crate::cmp::StateMeta<'_> = &mut state_lock.0.meta.write().unwrap();

		style.set_text_content(
			&styles
				.read()
				.unwrap()
				.iter()
				.fold(String::new(), |acc, (class, style)| acc + &format!(".{} {{ {} }}", class, style)),
		);
	}

	let body = document().body().unwrap();
	let first = body.child_nodes().item(0);

	update_node(&mut Element::from(body), first, Some(Node::from(new_node)));
	state_lock.0.meta.write().unwrap().dirty = false;
}

#[allow(clippy::option_unwrap_used)]
pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &'static crate::StateLock<'static, S>, mount: F) {
	let _ = std::mem::replace(&mut rw_lock.update_meta().mount, Box::new(mount));

	document().head().unwrap().append_child(&rw_lock.view_meta().style);
	// update(&rw_lock);
}
