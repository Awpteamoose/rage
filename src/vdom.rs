use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	js,
	console,
	traits::*,
	web::{
		document,
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
use std::sync::{Arc, RwLock};
use std::ops::{Deref, DerefMut};

pub struct Element {
	pub dom_reference: Option<DomElement>,

	pub tag: crate::primitives::Tag,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub event_handlers: Vec<crate::primitives::EventHandler>,
}

impl PartialEq for Element {
	fn eq(&self, other: &Self) -> bool {
		self.tag == other.tag &&
		self.children == other.children &&
		self.attributes == other.attributes
	}
}

impl Eq for Element {}

impl std::fmt::Debug for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{tag}{attributes}>{children}</div>",
			tag=self.tag.as_static(),
			attributes=self.attributes.iter().fold(String::new(), |acc, (key, value)|
				acc + &format!(r#" "{key}"="{value}""#, key=key, value=value)
			),
			children=self.children.iter().fold(String::new(), |acc, child|
				acc + &format!("{:?}\n", child)
			),
		)
	}
}

impl Element {
	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure)]
	pub fn render(&mut self) -> DomElement {
		// console!(log, "RENDER", self.tag.as_static());

		let element = document().create_element(self.tag.as_static()).unwrap();

		for child in &mut self.children {
			element.append_child(child.dom_node());
		}

		for (name, value) in self.attributes.iter() {
			element.set_attribute(name, value).unwrap();
		}

		if self.tag == crate::primitives::Tag::input {
			// console!(log, "REASSIGN HANDLERS");
		}
		let event_handlers = std::mem::replace(&mut self.event_handlers, vec![]);
		for handler in event_handlers.into_iter() {
			__event_idents![__event_listeners, handler, element];
		}

		element
	}

	pub fn dom_node(&mut self) -> &DomElement {
		// console!(log, "dom_node", line!());
		if self.dom_reference.is_none() {
			self.dom_reference = Some(self.render());
		}
		// console!(log, "dom_node", line!());

		self.dom_reference.as_ref().unwrap()
	}
}

impl Element {
	pub fn new(
		tag: crate::primitives::Tag,
		children: Vec<Self>,
		attributes: HashMap<String, String>,
		event_handlers: Vec<crate::primitives::EventHandler>,
	) -> Self {
		Self {
			dom_reference: None,
			tag,
			children,
			attributes,
			event_handlers,
		}
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree(parent_dom: &DomElement, old: Option<&mut Element>, new: Option<&mut Element>) {
	match (old, new) {
		(None, Some(new)) => {
			// console!(log, "append on compare");
			parent_dom.append_child(new.dom_node());
		},
		(Some(old), None) => {
			// console!(log, "remove on compare");
			let _ = parent_dom.remove_child(old.dom_node()).unwrap();
		},
		(Some(old), Some(new)) => {
			if old == new {
				// let (mut old_data, mut new_data) = (old, new);
				// console!(log, "equal");
				// TODO: just nest and pass the references
				// old_data.dom_reference = new_data.dom_reference.clone();
				// return;
			}

			if old.tag != new.tag {
				let _ = parent_dom.replace_child(new.dom_node(), old.dom_node()).unwrap();
			}

			new.dom_reference = old.dom_reference.take();

			if new.attributes != old.attributes {
				// console!(log, "attributes");
				let _ = new.dom_node();
				let new_dom = new.dom_reference.as_ref().unwrap();

				for (name, value) in new.attributes.iter() {
					new_dom.set_attribute(name, value).unwrap();
				}
			}

			let children_number = usize::max(old.children.len(), new.children.len());

			let new_parent = new.dom_reference.as_ref().unwrap();

			// console!(log, "dom_ref", &old_data.dom_reference, &new_data.dom_reference);

			for id in 0..children_number {
				patch_tree(new_parent, old.children.get_mut(id), new.children.get_mut(id));
			}
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default + 'static>(state_lock: &'static crate::cmp::StateLock<S>) {
	// console!(log, "UPDATE START");
	// console!(log, "RERENDER");
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

	let mut meta = state_lock.0.meta.write().unwrap();

	let old_node = &mut meta.vdom;
	// console!(log, format!("{:?} <-> {:?}", old_node, new_node));
	let element = document().get_element_by_id("__rage__").unwrap();
	patch_tree(&element, Some(old_node), Some(&mut new_node));
	meta.vdom = new_node;

	meta.dirty = false;
	// console!(log, "UPDATE END");
}

#[allow(clippy::option_unwrap_used)]
pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &'static crate::StateLock<S>, mount: F) {
	let mut meta = rw_lock.update_meta();
	let dom_ref = meta.vdom.render();
	dom_ref.set_attribute("id", "__rage__").unwrap();
	meta.vdom.dom_reference = Some(dom_ref);
	document().body().unwrap().append_child(meta.vdom.dom_reference.as_ref().unwrap());
	let _ = std::mem::replace(&mut meta.mount, Box::new(mount));
	document().head().unwrap().append_child(&meta.style);
}
