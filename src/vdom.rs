use crate::{
	cmp::{StateLock, StateLockKey, StateMeta},
	primitives::{EventHandler, Tag},
};
use matches::matches;
use std::collections::{HashMap, HashSet};
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

pub struct Element {
	pub dom_reference: Option<DomNode>,

	pub tag: Tag,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub event_handlers: Option<Vec<EventHandler>>,
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
	pub fn new(tag: Tag, children: Vec<Self>, attributes: HashMap<String, String>, event_handlers: Vec<EventHandler>) -> Self {
		Self {
			dom_reference: None,
			tag,
			children,
			attributes,
			event_handlers: Some(event_handlers),
			listener_handles: Vec::new(),
		}
	}

	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure)]
	pub fn render(&mut self) -> DomNode {
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
		for child in &mut self.children {
			child.attach_handlers();
		}
		// if already attached - just return
		let event_handlers = if let Some(x) = self.event_handlers.take() { x } else { return; };
		let element = self.dom_reference.as_ref().unwrap();
		let _ = std::mem::replace(
			&mut self.listener_handles,
			event_handlers
				.into_iter()
				.map(|handler| __event_idents![__event_listeners, handler, element])
				.collect(),
		);
	}

	pub fn detach_handlers(&mut self) {
		for child in &mut self.children {
			child.detach_handlers();
		}
		if self.listener_handles.is_empty() { return; }
		let listener_handles = std::mem::replace(&mut self.listener_handles, vec![]);
		for handle in listener_handles {
			handle.remove();
		}
	}

	pub fn dom_node(&mut self) -> &DomNode {
		if self.dom_reference.is_none() {
			self.dom_reference = Some(self.render());
		}

		self.dom_reference.as_ref().unwrap()
	}
}

impl<S: Into<String>> From<S> for Element {
	fn from(s: S) -> Self {
		Self::new(Tag::text_node(s.into()), Vec::new(), HashMap::new(), Vec::new())
	}
}

fn fix_inputs(node: &DomNode, elem: &Element) {
	// inputs are retarded
	if matches!(elem.tag, Tag::input) {
		if let Some(input_type) = elem.attributes.get("type") {
			match input_type as &str {
				"checkbox" | "radio" => {
					let checked = elem.attributes.get("checked").is_some();
					js!(@{node}.checked = @{checked});
				},
				"text" => {
					if let Some(value) = elem.attributes.get("value") {
						js!(@{node}.value = @{value});
					}
				},
				_ => {},
			}
		}
	}
}

// TODO: review, rewrite, avoid unwraps, avoid clones, avoid retardation
#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree(parent_dom: &DomElement, old: Option<&mut Element>, new: Option<&mut Element>) {
	match (old, new) {
		(None, Some(new)) => {
			parent_dom.append_child(new.dom_node());

			if new.children.is_empty() {
				new.attach_handlers();
				fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
				return;
			}

			let new_parent = DomElement::try_from(new.dom_reference.as_ref().unwrap().as_ref()).unwrap();
			for child in &mut new.children {
				patch_tree(&new_parent, None, Some(child));
			}

			new.attach_handlers();
		},
		(Some(old), None) => {
			old.detach_handlers();
			let _ = parent_dom.remove_child(old.dom_node()).unwrap();
		},
		(Some(old), Some(new)) => {
			old.detach_handlers();

			if old == new {
				new.dom_reference = old.dom_reference.take();
				let children_number = new.children.len();
				if children_number == 0 {
					new.attach_handlers();
					fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
					return;
				}

				let new_parent = DomElement::try_from(new.dom_reference.as_ref().unwrap().as_ref()).unwrap();
				for id in 0..children_number {
					patch_tree(&new_parent, old.children.get_mut(id), new.children.get_mut(id));
				}

				new.attach_handlers();
				fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
				return;
			}

			if (old.tag != new.tag) || matches!(new.tag, Tag::text_node(_)) {
				let _ = parent_dom.replace_child(new.dom_node(), old.dom_node()).unwrap();
				new.attach_handlers();
				fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
				return;
			}

			new.dom_reference = old.dom_reference.take();

			let new_dom = DomElement::try_from(new.dom_reference.as_ref().unwrap().as_ref()).unwrap();

			if new.attributes != old.attributes {
				let all_attributes: HashSet<_> = old.attributes.keys().chain(new.attributes.keys()).collect();
				for name in all_attributes {
					match new.attributes.get(name) {
						// still in new
						Some(new_value) => {
							if let Some(old_value) = old.attributes.get(name) {
								// changed
								if old_value != new_value {
									new_dom.set_attribute(name, new_value).unwrap();
								}
							} else {
								// wasn't present
								new_dom.set_attribute(name, new_value).unwrap();
							}
						},
						// removed in new
						None => new_dom.remove_attribute(name),
					}
				}
			}

			for id in 0..usize::max(old.children.len(), new.children.len()) {
				patch_tree(&new_dom, old.children.get_mut(id), new.children.get_mut(id));
			}

			new.attach_handlers();
			fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default>(state_lock_key: &'static impl StateLockKey<S>) {
	// console!(log, "UPDATE START");
	state_lock_key.lock(|state_lock| {
		state_lock.view_meta().styles.borrow_mut().clear();

		let mut new_vdom = (state_lock.view_meta().mount)();
		let mut meta = state_lock.update_meta();

		meta.style.set_text_content(
			&meta
				.styles
				.borrow()
				.iter()
				.fold(String::new(), |acc, (class, style)| acc + &format!(".{} {{ {} }}", class, style)),
		);

		let element = document().get_element_by_id("__rage__").unwrap();
		patch_tree(&element, Some(&mut meta.vdom), Some(&mut new_vdom));
		meta.vdom = new_vdom;

		meta.dirty = false;
	});
	// console!(log, "UPDATE END");
}

#[allow(clippy::option_unwrap_used)]
pub fn mount<S: Default, F: Fn() -> Element + 'static>(state_lock_key: &'static impl StateLockKey<S>, mount: F) {
	state_lock_key.update_meta(|meta| {
		let dom_node = meta.vdom.dom_node();
		DomElement::try_from(dom_node.as_ref())
			.expect("bad node")
			.set_attribute("id", "__rage__")
			.expect("can't set attribute");
		document().body().unwrap().append_child(dom_node);
		let _ = std::mem::replace(&mut meta.mount, Box::new(mount));
		document().head().unwrap().append_child(&meta.style);
	})
}
