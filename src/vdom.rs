use crate::{
	cmp::{State, STATE},
	primitives::{EventHandler, XmlElement, html::Tag as HtmlTag},
};
use matches::matches;
use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
};
use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
	traits::*,
	unstable::{TryFrom, TryInto},
	web::{document, Element as DomElement, Node as DomNode, event::ConcreteEvent},
};
use strum::AsStaticRef;

macro_rules! cb_map {
	($callbacks: ident, skip, $($name: ident),+$(,)*) => {
		#[derive(Default)]
		#[allow(dead_code, non_snake_case, missing_debug_implementation)]
		pub struct $callbacks {
			pub id: u32,
			$(
				pub $name: HashMap<u32, Box<dyn Fn(&stdweb::web::event::$name)>>,
			)+
		}
	};
}

__event_idents![cb_map, Callbacks, skip];

thread_local! {
	pub static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
}

pub struct Element {
	pub dom_reference: Option<DomNode>,

	pub xml_element: XmlElement,
	pub children: Vec<Element>,
	pub attributes: HashMap<String, String>,
	pub event_handlers: Option<Vec<EventHandler>>,
	pub callback_id: u32,
}

impl PartialEq for Element {
	fn eq(&self, other: &Self) -> bool {
		self.xml_element == other.xml_element && self.children == other.children && self.attributes == other.attributes
	}
}

impl Eq for Element {}

// impl std::fmt::Debug for Element {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "<{tag}{attributes}>{children}</div>",
//             tag = self.tag.as_static(),
//             attributes = self.attributes.iter().fold(String::new(), |acc, (key, value)| acc + &format!(
//                 r#" "{key}"="{value}""#,
//                 key = key,
//                 value = value
//             )),
//             children = self
//                 .children
//                 .iter()
//                 .fold(String::new(), |acc, child| acc + &format!("{:?}\n", child)),
//         )
//     }
// }

impl Element {
	pub fn new(xml_element: XmlElement, children: Vec<Self>, attributes: HashMap<String, String>, event_handlers: Vec<EventHandler>) -> Self {
		Self {
			dom_reference: None,
			xml_element,
			children,
			attributes,
			event_handlers: Some(event_handlers),
			callback_id: CALLBACKS.with(move |c| {
				let mut callbacks = c.borrow_mut();
				let id = callbacks.id;
				callbacks.id = callbacks.id.overflowing_add(1).0;
				id
			}),
		}
	}

	#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, clippy::redundant_closure, unused_must_use)]
	pub fn render(&mut self) -> DomNode {
		let element = match &self.xml_element {
			XmlElement::TextNode(s) => return document().create_text_node(s).into(),
			XmlElement::Raw(s) => DomElement::try_from(DomNode::from_html(s.trim()).unwrap()).unwrap(),
			XmlElement::Html(tag) => document().create_element(tag.as_static()).unwrap(),
			XmlElement::Svg(tag) => document().create_element_ns("http://www.w3.org/2000/svg", tag.as_static()).unwrap(),
		};

		for child in &mut self.children {
			element.append_child(child.dom_node());
		}

		for (name, value) in self.attributes.iter() {
			element.set_attribute(name, value);
		}

		js!(@{&element}.__rage_event_callback = @{self.callback_id});

		element.into()
	}

	pub fn attach_handlers(&mut self) {
		for child in &mut self.children {
			child.attach_handlers();
		}
		// if already attached - just return
		let event_handlers = if let Some(x) = self.event_handlers.take() { x } else { return; };
		if event_handlers.is_empty() { return; };
		CALLBACKS.with(move |c| {
			let mut callbacks = c.borrow_mut();
			for handler in event_handlers {
				macro_rules! handle {
					(skip, skip, $($name: ident),+$(,)*) => {
						match handler {
							$(EventHandler::$name(handler) => { let _ = callbacks.$name.insert(self.callback_id, handler); },)+
						}
					};
				}

				__event_idents!(handle, skip, skip)
			}
		});
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
		Self::new(XmlElement::TextNode(s.into()), Vec::new(), HashMap::new(), Vec::new())
	}
}

fn fix_inputs(node: &DomNode, elem: &Element) {
	for child in &elem.children {
		if let Some(node) = &child.dom_reference {
			fix_inputs(node, child);
		}
	}

	// inputs are retarded
	match elem.xml_element {
		XmlElement::Html(HtmlTag::input) => {
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
		},
		XmlElement::Html(HtmlTag::select) | XmlElement::Html(HtmlTag::textarea) => {
			if let Some(value) = elem.attributes.get("value") {
				js!(@{node}.value = @{value});
			}
		},
		_ => {},
	}
}

// TODO: review, rewrite, avoid unwraps, avoid clones, avoid retardation
#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, unused_must_use)]
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
			fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
		},
		(Some(old), None) => {
			parent_dom.remove_child(old.dom_node());
		},
		(Some(old), Some(new)) => {
			if old == new {
				new.dom_reference = old.dom_reference.take();
				if new.callback_id != old.callback_id {
					js!(@{&new.dom_reference}.__rage_event_callback = @{new.callback_id});
				}
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

			if (old.xml_element != new.xml_element) || matches!(new.xml_element, XmlElement::TextNode(_)) {
				let new_dom_node = new.dom_node();
				let old_dom_node = old.dom_node();
				parent_dom.replace_child(new_dom_node, old_dom_node);
				new.attach_handlers();
				fix_inputs(&new.dom_reference.as_ref().unwrap(), &new);
				return;
			}

			new.dom_reference = old.dom_reference.take();
			if new.callback_id != old.callback_id {
				js!(@{&new.dom_reference}.__rage_event_callback = @{new.callback_id});
			}

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
									new_dom.set_attribute(name, new_value);
								}
							} else {
								// wasn't present
								new_dom.set_attribute(name, new_value);
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
pub fn update(_: f64) {
	// console!(log, "UPDATE START");
	STATE.with(|lock| {
		*lock.borrow().dirty.borrow_mut() = false;
		CALLBACKS.with(|c| *c.borrow_mut() = Callbacks::default());
		let mut new_vdom = (lock.borrow().render)();
		let mut meta = lock.borrow_mut();

		let element = document().get_element_by_id("__rage__").unwrap();
		patch_tree(&element, Some(&mut meta.vdom), Some(&mut new_vdom));
		meta.vdom = new_vdom;
	});
	// console!(log, "UPDATE END");
}

#[allow(clippy::option_unwrap_used, unused_must_use)]
pub fn mount<F: Fn() -> Element + 'static>(mount: F) {
	STATE.with(|lock| {
		let mut meta = lock.borrow_mut();
		let dom_node = meta.vdom.dom_node();
		DomElement::try_from(dom_node.as_ref())
			.expect("bad node")
			.set_attribute("id", "__rage__");

		macro_rules! attach_cb {
			(skip, skip, $($name: ident),+$(,)*) => {{
				$(
					let cb = move |e: stdweb::web::event::$name| {
						let ids: Vec<u32> = js!{return @{&e}.composedPath().reduce((acc, node) => {
							const id = node.__rage_event_callback;
							if (id) acc.push(id);
							return acc;
						}, []);}.try_into().unwrap();
						CALLBACKS.with(move |c| {
							for id in ids {
								if let Some(f) = c.borrow().$name.get(&id) {
									return f(&e);
								}
							}
						});
					};
					js!(document.addEventListener(@{stdweb::web::event::$name::EVENT_TYPE}, @{cb}, { capture: true }));
				)+
			}};
		}

		__event_idents![attach_cb, skip, skip];

		document().body().unwrap().append_child(dom_node);
		std::mem::replace(&mut meta.render, Box::new(mount));
	});
	update(0.);
}
