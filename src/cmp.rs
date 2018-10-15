use crate::dom;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use stdweb::web::{document, Element};

#[allow(missing_debug_implementations)]
pub struct StateLock<S: Default> {
	pub style: Element,
	pub mount: Cmp<S>,
	pub styles: Rc<RefCell<HashMap<String, String>>>,
	pub state: S,
}

impl<S: Default> Default for StateLock<S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		Self {
			style: document().create_element("style").unwrap(),
			mount: Cmp::new(|_| document().create_element("div").unwrap()),
			styles: Rc::default(),
			state: S::default(),
		}
	}
}

impl<S: Default> StateLock<S> {
	pub fn update(state_rc: &mut StateRc<S>, f: impl Fn(&mut S)) {
		f(&mut state_rc.borrow_mut().state);
		dom::update(state_rc);
	}
}

pub type StateRc<S> = Rc<RefCell<StateLock<S>>>;

#[allow(missing_debug_implementations)]
pub struct Cmp<T: Default>(pub Box<dyn Fn(StateRc<T>) -> Element>);

impl<T: Default> Cmp<T> {
	pub fn new(f: impl 'static + Fn(StateRc<T>) -> Element) -> Self {
		Cmp(Box::new(f))
	}
}
