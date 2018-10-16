// node and element traits
// each node stores a reference to their real dom node
// diff as I do right now, except return update pairs, to_delete and to_append and then work on real dom
// ???
// fast as fukkkkkkkkkkk

use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	js,
	console,
	web::{
		Element as DomElement,
		Node as DomNode,
	},
};
use std::collections::{
	HashMap,
	HashSet,
};

#[derive(PartialEq, Eq)]
pub struct Element<'a> {
	// dom_reference: Option<DomNode>,

	pub tag: crate::primitives::Tag,
	pub parent: Option<&'a Element<'a>>,
	pub children: Vec<Element<'a>>,
	pub attributes: HashMap<String, String>,
}

impl<'a> Element<'a> {
	pub fn render(&mut self) -> DomElement {
		unimplemented!()
	}
}

// #[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
// fn update_attributes(old: &Element, new: &Element) {
//     let old_attrs = old.get_attribute_names();
//     let new_attrs = new.get_attribute_names();

//     for attr in old_attrs.into_iter() {
//         if old.get_attribute(&attr).is_some() && new.get_attribute(&attr).is_none() {
//             old.remove_attribute(&attr);
//         }
//     }

//     for attr in new_attrs.into_iter() {
//         if let (Some(new_value), Some(old_value)) = (new.get_attribute(&attr), old.get_attribute(&attr)) {
//             if new_value != old_value {
//                 let _ = old.set_attribute(&attr, &new_value);
//             }
//         }
//     }
// }

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree<'a>(old: Option<&mut Element<'a>>, new: Option<&mut Element<'a>>) {
	match (old, new) {
		(None, Some(new)) => {
			console!(log, "append on compare");
			// TODO: insert into dom
			// let _ = parent.children.push(new);
		},
		(Some(old), None) => {
			console!(log, "remove on compare");
			// TODO: remove from dom
			// let _ = parent.children.remove(parent.children.binary_search(&old).unwrap());
		},
		(Some(old), Some(new)) => {
			if old == new { return; }

			if old.children.is_empty() || new.children.is_empty() {
				// TODO: swap in dom
				// let _ = parent.children.remove(&old);
				// let _ = parent.children.insert(new);
				// return;
			}

			// TODO: update attributes

			for id in 0..usize::max(old.children.len(), new.children.len()) {
				let (old_node, new_node) = (old.children.get_mut(id), new.children.get_mut(id));
				patch_tree(old_node, new_node);
			}
		},
		_ => {},
	}
}

// #[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
// pub fn update<S: Default + 'static>(state_lock: &crate::cmp::StateLock<S>) {
//     console!(log, "RERENDER");
//     state_lock.view_meta().styles.write().unwrap().clear();

//     let new_node = (state_lock.view_meta().mount)();

//     {
//         let crate::cmp::StateMeta { styles, style, .. }: &mut crate::cmp::StateMeta = &mut state_lock.0.meta.write().unwrap();

//         style.set_text_content(
//             &styles
//                 .read()
//                 .unwrap()
//                 .iter()
//                 .fold(String::new(), |acc, (class, style)| acc + &format!(".{} {{ {} }}", class, style)),
//         );
//     }

//     let body = document().body().unwrap();
//     let first = body.child_nodes().item(0);

//     update_node(&mut Element::from(body), first, Some(Node::from(new_node)));
//     state_lock.0.meta.write().unwrap().dirty = false;
// }

// #[allow(clippy::option_unwrap_used)]
// pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &crate::StateLock<S>, mount: F) {
//     let _ = std::mem::replace(&mut rw_lock.update_meta().mount, Box::new(mount));

//     document().head().unwrap().append_child(&rw_lock.view_meta().style);
//     // update(&rw_lock);
// }
