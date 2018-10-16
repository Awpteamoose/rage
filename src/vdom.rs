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
use std::sync::{Arc, RwLock};
use std::ops::{Deref, DerefMut};

pub struct ElementData {
	pub dom_reference: Option<DomNode>,

	pub tag: crate::primitives::Tag,
	pub parent: Option<Element>,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub event_handlers: Vec<crate::primitives::EventHandler>,
}

#[derive(Clone)]
pub struct Element(pub Arc<RwLock<ElementData>>);

impl PartialEq for Element {
	fn eq(&self, other_lock: &Self) -> bool {
		let me = self.read_data();
		let other = other_lock.read_data();

		(me.tag == other.tag) &&
		(
			(me.parent.is_none() == other.parent.is_none()) ||
			(
				(me.parent.is_some() == other.parent.is_some()) &&
				(&me.parent.as_ref().expect(&format!("{}:{}", file!(), line!())) as &Element == &other.parent.as_ref().expect(&format!("{}:{}", file!(), line!())) as &Element)
			)
		) &&
		(me.children == other.children) &&
		(me.attributes == other.attributes)
	}
}

impl Eq for Element {}

impl std::fmt::Debug for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let me = self.read_data();

		write!(f, "<{tag}{attributes}>{children}</div>",
			tag=me.tag.as_static(),
			attributes=me.attributes.iter().fold(String::new(), |acc, (key, value)|
				acc + &format!(r#" "{key}"="{value}""#, key=key, value=value)
			),
			children=me.children.iter().fold(String::new(), |acc, child|
				acc + &format!("{:?}\n", child)
			),
		)
	}
}

impl ElementData {
	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure)]
	pub fn render(&mut self) -> DomNode {
		// console!(log, "RENDER", self.tag.as_static());

		let element = stdweb::web::document().create_element(self.tag.as_static()).expect(&format!("{}:{}", file!(), line!()));

		for child in &mut self.children {
			element.append_child(child.write_data().dom_node());
		}

		for (name, value) in self.attributes.iter() {
			element.set_attribute(name, value).expect(&format!("{}:{}", file!(), line!()));
		}

		if self.tag == crate::primitives::Tag::input {
			// console!(log, "REASSIGN HANDLERS");
		}
		let event_handlers = std::mem::replace(&mut self.event_handlers, vec![]);
		for handler in event_handlers.into_iter() {
			__event_idents![__event_listeners, handler, element];
		}

		element.into()
	}

	pub fn parent_dom(&self) -> DomElement {
		if let Some(parent) = self.parent.as_ref() {
			// console!(log, "parent_dom", line!());
			let parent_data = parent.read_data();
			// console!(log, "parent_dom", line!(), &parent_data.dom_reference);
			let parent_data_dom_reference = parent_data.dom_reference.as_ref().expect(&format!("{}:{}", file!(), line!()));
			// console!(log, "parent_dom", line!());
			DomElement::try_from(parent_data_dom_reference.clone()).expect(&format!("{}:{}", file!(), line!()))
		} else {
			// console!(log, "parent_dom", line!());
			stdweb::web::document().body().expect(&format!("{}:{}", file!(), line!())).into()
		}
	}

	pub fn dom_node(&mut self) -> &DomNode {
		// console!(log, "dom_node", line!());
		if self.dom_reference.is_none() {
			self.dom_reference = Some(self.render());
		}
		// console!(log, "dom_node", line!());

		self.dom_reference.as_ref().expect(&format!("{}:{}", file!(), line!()))
	}
}

impl Element {
	pub fn new(
		tag: crate::primitives::Tag,
		children: Vec<Element>,
		attributes: HashMap<String, String>,
		event_handlers: Vec<crate::primitives::EventHandler>,
	) -> Self {
		let me = Element(Arc::new(RwLock::new(ElementData {
			dom_reference: None,
			parent: None,
			tag,
			children,
			attributes,
			event_handlers,
		})));

		for child in &mut me.0.write().expect(&format!("{}:{}", file!(), line!())).children {
			child.0.write().expect(&format!("{}:{}", file!(), line!())).parent = Some(me.clone());
		}

		me
	}

	pub fn read_data<'a>(&'a self) -> impl Deref<Target = ElementData> + 'a {
		self.0.read().expect(&format!("{}:{}", file!(), line!()))
	}

	pub fn write_data<'a>(&'a self) -> impl DerefMut<Target = ElementData> + 'a {
		self.0.write().expect(&format!("{}:{}", file!(), line!()))
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn patch_tree(old: Option<&Element>, new: Option<&Element>) {
	match (old, new) {
		(None, Some(new)) => {
			// console!(log, "append on compare");
			let parent = new.read_data().parent_dom();
			parent.append_child(new.write_data().dom_node());
		},
		(Some(old), None) => {
			// console!(log, "remove on compare");
			let parent = old.read_data().parent_dom();
			let _ = parent.remove_child(old.write_data().dom_node());
		},
		(Some(old), Some(new)) => {
			// if old == new {
			//     let (mut old_data, mut new_data) = (old.write_data(), new.write_data());
			//     console!(log, "equal");
			//     old_data.dom_reference = new_data.dom_reference.clone();
			//     return;
			// }

			let children_number = {
				let (mut old_data, mut new_data) = (old.write_data(), new.write_data());

				if old_data.children.is_empty() || new_data.children.is_empty() {
					// console!(log, "replace", line!());
					let parent = new_data.parent_dom();
					// console!(log, "replace", line!());
					let old_node = old_data.dom_node();
					// console!(log, "replace", line!());
					let new_node = new_data.dom_node();
					// console!(log, "replace with nodes", line!(), old_node, new_node);
					let _ = parent.replace_child(new_node, old_node).expect(&format!("{}:{}", file!(), line!()));
					// console!(log, "replaced");
					return;
				}

				new_data.dom_reference = old_data.dom_reference.clone();

				if new_data.attributes != old_data.attributes {
					// console!(log, "attributes");
					let new_dom = DomElement::try_from(new_data.dom_node().clone()).expect(&format!("{}:{}", file!(), line!()));

					for (name, value) in new_data.attributes.iter() {
						new_dom.set_attribute(name, value).expect(&format!("{}:{}", file!(), line!()));
					}
				}

				usize::max(old_data.children.len(), new_data.children.len())
			};

			let (old_data, new_data) = (old.read_data(), new.read_data());

			// console!(log, "dom_ref", &old_data.dom_reference, &new_data.dom_reference);

			for id in 0..children_number {
				let (old_node, new_node) = (old_data.children.get(id), new_data.children.get(id));
				patch_tree(old_node, new_node);
			}
		},
		_ => {},
	}
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
pub fn update<S: Default + 'static>(state_lock: &'static crate::cmp::StateLock<S>) {
	// console!(log, "UPDATE START");
	// console!(log, "RERENDER");
	state_lock.view_meta().styles.write().expect(&format!("{}:{}", file!(), line!())).clear();

	let new_node = (state_lock.view_meta().mount)();

	{
		let crate::cmp::StateMeta { styles, style, .. }: &mut crate::cmp::StateMeta = &mut state_lock.0.meta.write().expect(&format!("{}:{}", file!(), line!()));

		style.set_text_content(
			&styles
				.read()
				.expect(&format!("{}:{}", file!(), line!()))
				.iter()
				.fold(String::new(), |acc, (class, style)| acc + &format!(".{} {{ {} }}", class, style)),
		);
	}

	let mut meta = state_lock.0.meta.write().expect(&format!("{}:{}", file!(), line!()));

	let old_node = &mut meta.vdom;
	// console!(log, format!("{:?} <-> {:?}", old_node, new_node));
	patch_tree(Some(old_node), Some(&new_node));
	meta.vdom = new_node;

	meta.dirty = false;
	// console!(log, "UPDATE END");
}

#[allow(clippy::option_unwrap_used)]
pub fn mount<S: Default + 'static, F: Fn() -> Element + 'static + Send + Sync>(rw_lock: &'static crate::StateLock<S>, mount: F) {
	let mut meta = rw_lock.update_meta();
	let dom_ref = meta.vdom.write_data().render();
	meta.vdom.write_data().dom_reference = Some(dom_ref);
	stdweb::web::document().body().expect(&format!("{}:{}", file!(), line!())).append_child(meta.vdom.read_data().dom_reference.as_ref().expect(&format!("{}:{}", file!(), line!())));
	let _ = std::mem::replace(&mut meta.mount, Box::new(mount));
	stdweb::web::document().head().expect(&format!("{}:{}", file!(), line!())).append_child(&meta.style);
}
