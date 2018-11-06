use crate::{
	vdom::Element,
	primitives::{
		XmlElement,
		html::Tag as HtmlTag,
	},
};
use maplit::*;
use std::{
	cell::RefCell,
	ops::{Deref, DerefMut},
	rc::Rc,
};
use stdweb::{__internal_console_unsafe, __js_raw_asm, _js_impl, console, js};

#[allow(missing_debug_implementations)]
pub struct State {
	pub render: Box<dyn Fn() -> Element>,
	pub dirty: RefCell<bool>,
	pub vdom: Element,
}

thread_local! {
	pub static STATE: RefCell<State> = RefCell::new(State {
		render: Box::new(|| unreachable!()),
		dirty: RefCell::new(true),
		vdom: Element::new(XmlElement::Html(HtmlTag::div), children!["Loading..."], attrs![], events![]),
	});
}

#[derive(Debug, Default)]
pub struct Tracked<T>(Rc<RefCell<T>>);
impl<T> Tracked<T> {
	pub fn new(state: T) -> Self {
		Tracked(Rc::new(RefCell::new(state)))
	}

	pub fn view<'a>(&'a self) -> impl Deref<Target = T> + 'a {
		self.0.borrow()
	}

	pub fn update<'a>(&'a self) -> impl DerefMut<Target = T> + 'a {
		STATE.with(|state| {
			let state = state.borrow();
			let mut dirty = state.dirty.borrow_mut();
			if !*dirty {
				*dirty = true;
				stdweb::web::window().request_animation_frame(crate::vdom::update);
			}
		});
		self.0.borrow_mut()
	}
}

impl<T> Clone for Tracked<T> {
	fn clone(&self) -> Self {
		Tracked(Rc::clone(&self.0))
	}
}

pub trait Component {
	fn render(&self) -> Element;
}
