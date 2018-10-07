use crate::{Component, StateRc, State};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use stdweb::{
	traits::*,
	web::{document, event, Node},
};

type OnClick = Box<dyn FnMut(event::ClickEvent)>;
type OnClickRc = Option<Rc<RefCell<OnClick>>>;
type Attributes = HashMap<String, String>;
type Children = Vec<Box<dyn Component>>;

struct Primitive {
	children: Children,
	attributes: Attributes,
	on_click: OnClickRc,
}

macro_rules! primitive {
	($name: ident, $tag: expr) => {
		pub struct $name(Primitive);

		impl $name {
			pub fn new() -> Self {
				$name(Primitive { attributes: Attributes::new(), children: Children::new(), on_click: None })
			}

			pub fn attributes(mut self, attributes: Attributes) -> Self {
				self.0.attributes = attributes;
				self
			}

			pub fn children(mut self, children: Children) -> Self {
				self.0.children = children;
				self
			}

			pub fn on_click(mut self, on_click: impl FnMut(event::ClickEvent) + 'static) -> Self {
				self.0.on_click = Some(Rc::new(RefCell::new(Box::new(on_click))));
				self
			}
		}

		impl Component for $name {
			fn render(&mut self, mut state: StateRc) -> Node {
				let element = document().create_element($tag).unwrap();

				for child in self.0.children.iter_mut() {
					element.append_child(&child.render(Rc::clone(&state)));
				}

				for (name, value) in self.0.attributes.iter() {
					element.set_attribute(name, value).unwrap();
				}

				if let Some(ref cb) = self.0.on_click {
					// TODO: maybe use it and then unhook events on re-render
					let on_click = Rc::clone(cb);
					let _handle = element.add_event_listener(move |e: event::ClickEvent| {
						let f: &mut OnClick = &mut on_click.borrow_mut();
						f(e)
					});
				}

				element.into()
			}
			fn children(&mut self) -> &mut Children { &mut self.0.children }
			fn attributes(&mut self) -> &mut Attributes { &mut self.0.attributes }
		}
	};
}

primitive!(Div, "div");

impl Component for String {
	fn render(&mut self, _state: StateRc) -> Node {
		document().create_text_node(&self).into()
	}
}

impl Component for &str {
	fn render(&mut self, _state: StateRc) -> Node {
		document().create_text_node(&self.to_string()).into()
	}
}

impl<T: ToString, F: Fn(&State) -> T> Component for F {
	fn render(&mut self, state: StateRc) -> Node {
		document().create_text_node(&self(&state.borrow().state).to_string()).into()
	}
}
