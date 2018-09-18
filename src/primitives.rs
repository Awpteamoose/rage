use crate::{Component, StateRc};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use stdweb::{
	traits::*,
	web::{document, event, Node},
};

type ClickOption = Option<Box<dyn FnMut(event::ClickEvent)>>;
type OnClick = Rc<RefCell<ClickOption>>;

struct Primitive {
	children: Vec<Box<dyn Component>>,
	attributes: HashMap<String, String>,
	on_click: OnClick,
}

macro_rules! primitive {
	($name: ident, $tag: expr) => {
		pub struct $name(Primitive);

		impl $name {
			pub fn new(attributes: HashMap<String, String>, children: Vec<Box<dyn Component>>, on_click: OnClick) -> $name {
				$name(Primitive { attributes, children, on_click })
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

				let mut on_click = Rc::clone(&self.0.on_click);
				element.add_event_listener(move |e: event::ClickEvent| {
					if let Some(f) = &mut on_click.borrow_mut() as &mut ClickOption {
						f(e)
					}
				});

				element.into()
			}
			fn children(&mut self) -> &mut Vec<Box<dyn Component>> { &mut self.0.children }
			fn attributes(&mut self) -> &mut HashMap<String, String> { &mut self.0.attributes }
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

impl<T: ToString, F: Fn(StateRc) -> T> Component for F {
	fn render(&mut self, state: StateRc) -> Node {
		document().create_text_node(&self(state).to_string()).into()
	}
}
