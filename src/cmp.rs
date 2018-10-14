use std::{
	cell::RefCell,
	collections::HashMap,
	rc::Rc,
};
use stdweb::{
	web::{document, Element},
};
use crate::dom;

#[allow(missing_debug_implementations)]
pub struct StateLock<S: Default> {
	pub style: Element,
	pub mount: FnCmp<S>,
	pub styles: Rc<RefCell<HashMap<String, String>>>,
	pub state: S,
}

impl<S: Default> Default for StateLock<S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		Self {
			style: document().create_element("style").unwrap(),
			mount: FnCmp(Box::new(|_| document().create_element("div").unwrap())),
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
pub struct FnCmp<S: Default>(pub Box<dyn Fn(&StateLock<S>) -> Element>);

impl<S: Default> FnCmp<S> {
	pub fn new(f: impl 'static + Fn(&StateLock<S>) -> Element) -> Self {
		FnCmp(Box::new(f))
	}
}
