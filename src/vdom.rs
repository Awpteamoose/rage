use std::collections::HashMap;
use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
	traits::*,
	unstable::TryFrom,
	web::{document, Element as DomElement, Node as DomNode},
};
use strum::AsStaticRef;
use crate::{
	primitives::{Tag, EventHandler},
	cmp::{StateLock, StateMeta},
};
use matches::matches;

pub struct Element {
	pub dom_reference: Option<DomNode>,

	pub tag: Tag,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub event_handlers: Vec<EventHandler>,
	pub listener_handles: Vec<stdweb::web::EventListenerHandle>,
}

impl PartialEq for Element {
	fn eq(&self, other: &Self) -> bool {
		self.tag == other.tag && self.children == other.children && self.attributes == other.attributes
	}
}

impl Eq for Element {}

impl std::fmt::Debug for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"<{tag}{attributes}>{children}</div>",
			tag = self.tag.as_static(),
			attributes = self.attributes.iter().fold(String::new(), |acc, (key, value)| acc + &format!(
				r#" "{key}"="{value}""#,
				key = key,
				value = value
			)),
			children = self
				.children
				.iter()
				.fold(String::new(), |acc, child| acc + &format!("{:?}\n", child)),
		)
	}
}

impl Element {
	pub fn new(
		tag: Tag,
		children: Vec<Self>,
		attributes: HashMap<String, String>,
		event_handlers: Vec<EventHandler>,
	) -> Self {
		Self {
			dom_reference: None,
			tag,
			children,
			attributes,
			event_handlers,
			listener_handles: Vec::new(),
		}
	}

	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure)]
	pub fn render(&mut self) -> DomNode {
		// console!(log, "RENDER", self.tag.as_static());
		if let Tag::text_node(s) = &self.tag {
			return document().create_text_node(s).into();
		}

		let element = document().create_element(self.tag.as_static()).unwrap();

		for child in &mut self.children {
			element.append_child(child.dom_node());
		}

		for (name, value) in self.attributes.iter() {
			element.set_attribute(name, value).unwrap();
		}

		element.into()
	}

	pub fn attach_handlers(&mut self) {
		let element = self.dom_reference.as_ref().unwrap();
		let event_handlers = std::mem::replace(&mut self.event_handlers, vec![]);
		std::mem::replace(&mut self.listener_handles, event_handlers.into_iter().map(|handler| __event_idents![__event_listeners, handler, element]).collect());
	}

	pub fn detach_handlers(&mut self) {
		let listener_handles = std::mem::replace(&mut self.listener_handles, vec![]);
		for handle in listener_handles {
			handle.remove();
		}
	}

	pub fn dom_node(&mut self) -> &DomNode {
		// console!(log, "dom_node", line!());
		if self.dom_reference.is_none() {
			self.dom_reference = Some(self.render());
		}
		// console!(log, "dom_node", line!());

		self.dom_reference.as_ref().unwrap()
		// self.dom_reference.as_ref().unwrap()
	}
}

impl<S: Into<String>> From<S> for Element {
	fn from(s: S) -> Self {
		Self::new(Tag::text_node(s.into()), Vec::new(), HashMap::new(), Vec::new())
	}
}

// TODO: review, rewrite, avoid unwraps, avoid clones, avoid retardation
#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree(parent_dom: &DomElement, old: Option<&mut Element>, new: Option<&mut Element>) {
	match (old, new) {
		(None, Some(new)) => {
			// console!(log, "append on compare");
			parent_dom.append_child(new.dom_node());
			new.attach_handlers();
			if new.children.is_empty() { return; }
			let new_parent = DomElement::try_from(new.dom_reference.as_ref().unwrap().clone()).unwrap();
			for child in &mut new.children {
				patch_tree(&new_parent, None, Some(child));
			}
		},
		(Some(old), None) => {
			// console!(log, "remove on compare");
			let _ = parent_dom.remove_child(old.dom_node()).unwrap();
		},
		(Some(old), Some(new)) => {
			if old == new {
				// console!(log, "equal");
				new.dom_reference = old.dom_reference.take();
				old.detach_handlers();
				new.attach_handlers();
				let children_number = usize::max(old.children.len(), new.children.len());
				if children_number == 0 { return; }
				let new_parent = DomElement::try_from(new.dom_reference.as_ref().unwrap().clone()).unwrap();
				for id in 0..children_number {
					patch_tree(&new_parent, old.children.get_mut(id), new.children.get_mut(id));
				}
				return;
			}

			if (old.tag != new.tag) || matches!(new.tag, Tag::text_node(_)) {
				// console!(log, "simple replace", &parent_dom);
				let _ = parent_dom.replace_child(new.dom_node(), old.dom_node()).unwrap();
				old.detach_handlers();
				new.attach_handlers();
				return;
				// console!(log, format!("{:?}", parent_dom.replace_child(new.dom_node(), old.dom_node())));
			}

			new.dom_reference = old.dom_reference.take();
			old.detach_handlers();
			new.attach_handlers();

			if new.attributes != old.attributes {
				// console!(log, "attributes");
				let new_dom = DomElement::try_from(new.dom_node().clone()).unwrap();

				for (name, value) in new.attributes.iter() {
					new_dom.set_attribute(name, value).unwrap();
				}
			}

			let children_number = usize::max(old.children.len(), new.children.len());

			let new_parent = DomElement::try_from(new.dom_reference.as_ref().unwrap().clone()).unwrap();

			// console!(log, "dom_ref", &old_data.dom_reference, &new_data.dom_reference);

			for id in 0..children_number {
				patch_tree(&new_parent, old.children.get_mut(id), new.children.get_mut(id));
			}
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default + 'static>(state_lock: &'static StateLock<S>) {
	// console!(log, "UPDATE START");
	// console!(log, "RERENDER");
	state_lock.view_meta().styles.write().unwrap().clear();

	let mut new_node = (state_lock.view_meta().mount)();

	{
		let StateMeta { styles, style, .. }: &mut StateMeta = &mut state_lock.0.meta.write().unwrap();

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
pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &'static StateLock<S>, mount: F) {
	let meta: &mut StateMeta = &mut rw_lock.update_meta();
	meta.vdom.dom_reference = Some(meta.vdom.render());
	DomElement::try_from(meta.vdom.dom_reference.as_ref().unwrap().clone()).unwrap().set_attribute("id", "__rage__").unwrap();
	document().body().unwrap().append_child(meta.vdom.dom_reference.as_ref().unwrap());
	let _ = std::mem::replace(&mut meta.mount, Box::new(mount));
	document().head().unwrap().append_child(&meta.style);
}
