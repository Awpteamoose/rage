use maplit::*;
use std::{
	rc::Rc,
	cell::RefCell,
	ops::{Deref, DerefMut},
};
use crate::vdom::Element;

#[allow(missing_debug_implementations)]
pub struct State {
	pub render: Box<dyn Fn() -> Element>,
	pub dirty: bool,
	pub vdom: Element,
}

thread_local! {
	pub static STATE: RefCell<State> = RefCell::new(State {
		render: Box::new(|| unreachable!()),
		dirty: false,
		vdom: Element::new(crate::primitives::Tag::div, children!["Loading..."], attrs![], events![]),
	});
}

#[derive(Debug, Default)]
pub struct Tracked<T>(Rc<RefCell<T>>);
impl<T> Tracked<T> {
	pub fn new(state: T) -> Self {
		Tracked(Rc::new(RefCell::new(state)))
	}

	pub fn view<'a>(&'a self) -> impl Deref<Target=T> + 'a {
		self.0.borrow()
	}

	pub fn update<'a>(&'a self) -> impl DerefMut<Target=T> + 'a {
		STATE.with(|s| {
			let mut state = s.borrow_mut();
			if !state.dirty {
				state.dirty = true;
				let _ = stdweb::web::window().request_animation_frame(crate::vdom::update);
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
