use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	js,
	console,
	traits::*,
	web::{
		Element as DomElement,
		Node as DomNode,
	},
	unstable::TryFrom,
};
use std::collections::{
	HashMap,
	HashSet,
};
use maplit::*;
use strum::AsStaticRef;

pub struct Element {
	pub dom_reference: Option<DomNode>,

	pub tag: crate::primitives::Tag,
	pub parent: Option<&'static Element>,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub events: Vec<crate::primitives::EventHandler>,
}

impl PartialEq for Element {
	fn eq(&self, other: &Self) -> bool {
		self.tag == other.tag &&
		self.parent == other.parent &&
		self.children == other.children &&
		self.attributes == other.attributes
	}
}

impl Eq for Element {}

impl Element {
	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure)]
	pub fn render(&mut self) -> DomNode {
		let element = stdweb::web::document().create_element(self.tag.as_static()).unwrap();

		for child in &mut self.children {
			element.append_child(&child.render());
		}

		for (name, value) in self.attributes.iter() {
			element.set_attribute(name, value).unwrap();
		}

		let events = std::mem::replace(&mut self.events, vec![]);
		for handler in events.into_iter() {
			__event_idents![__event_listeners, handler, element];
		}

		element.into()
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree(old: Option<&mut Element>, new: Option<&mut Element>) {
	match (old, new) {
		(None, Some(new)) => {
			console!(log, "append on compare");
			new.parent.as_ref().unwrap().dom_reference.as_ref().unwrap().append_child(&new.render());
		},
		(Some(old), None) => {
			console!(log, "remove on compare");
			let _ = old.parent.as_ref().unwrap().dom_reference.as_ref().unwrap().remove_child(old.dom_reference.as_ref().unwrap()).unwrap();
		},
		(Some(old), Some(new)) => {
			new.dom_reference = old.dom_reference.clone();
			if old == new { return; }

			if old.children.is_empty() || new.children.is_empty() {
				let parent_dom = new.parent.as_ref().unwrap().dom_reference.as_ref().unwrap();
				let old_dom = old.dom_reference.as_ref().unwrap();

				if new.dom_reference.is_none() {
					new.dom_reference = Some(new.render());
				}
				let new_dom = new.dom_reference.as_ref().unwrap();
				let _ = parent_dom.replace_child(old_dom, new_dom).unwrap();
				return;
			}

			if new.attributes != old.attributes {
				if new.dom_reference.is_none() { new.dom_reference = Some(new.render()); }
				let new_dom = DomElement::try_from(new.dom_reference.as_ref().unwrap().clone()).unwrap();
				for (name, value) in new.attributes.iter() {
					new_dom.set_attribute(name, value).unwrap();
				}
			}

			for id in 0..usize::max(old.children.len(), new.children.len()) {
				let (old_node, new_node) = (old.children.get_mut(id), new.children.get_mut(id));
				patch_tree(old_node, new_node);
			}
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default + 'static>(state_lock: &'static crate::cmp::StateLock<S>) {
	console!(log, "RERENDER");
	state_lock.view_meta().styles.write().unwrap().clear();

	let mut new_node = (state_lock.view_meta().mount)();

	{
		let crate::cmp::StateMeta { styles, style, .. }: &mut crate::cmp::StateMeta = &mut state_lock.0.meta.write().unwrap();

		style.set_text_content(
			&styles
				.read()
				.unwrap()
				.iter()
				.fold(String::new(), |acc, (class, style)| acc + &format!(".{} {{ {} }}", class, style)),
		);
	}

	patch_tree(Some(&mut state_lock.0.meta.write().unwrap().vdom), Some(&mut new_node));

	state_lock.0.meta.write().unwrap().dirty = false;
}

#[allow(clippy::option_unwrap_used)]
pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &'static crate::StateLock<S>, mount: F) {
	let _ = std::mem::replace(&mut rw_lock.update_meta().mount, Box::new(mount));

	stdweb::web::document().head().unwrap().append_child(&rw_lock.view_meta().style);
}
