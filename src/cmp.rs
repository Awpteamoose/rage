use maplit::*;
use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	cell::RefCell,
};
use stdweb::web::{document, Element};

#[allow(missing_debug_implementations)]
pub struct StateMeta {
	pub style: Element,
	pub mount: Box<dyn Fn() -> crate::vdom::Element>,
	pub styles: RefCell<HashMap<String, String>>,
	pub dirty: bool,
	pub vdom: crate::vdom::Element,
}

#[allow(missing_debug_implementations)]
pub struct StateLock<S: Default> {
	meta: RefCell<StateMeta>,
	state: RefCell<S>,
}

impl<S: Default> Default for StateLock<S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		Self {
			state: RefCell::new(S::default()),
			meta: RefCell::new(StateMeta {
				style: document().create_element("style").unwrap(),
				mount: Box::new(|| unreachable!()),
				styles: RefCell::new(HashMap::default()),
				dirty: false,
				vdom: crate::vdom::Element::new(crate::primitives::Tag::div, children!["Loading..."], attrs![], events![]),
			}),
		}
	}
}

pub trait StateLockKey<S: Default> {
	fn lock<R>(&'static self, f: impl FnOnce(&StateLock<S>) -> R) -> R;

	fn update<R>(&'static self, f: impl FnOnce(&mut S) -> R) -> R;
	fn view<R>(&'static self, f: impl FnOnce(&S) -> R) -> R;

	fn update_meta<R>(&'static self, f: impl FnOnce(&mut StateMeta) -> R) -> R;
	fn view_meta<R>(&'static self, f: impl FnOnce(&StateMeta) -> R) -> R;
}

impl<S: Default> StateLockKey<S> for std::thread::LocalKey<StateLock<S>> {
	fn lock<R>(&'static self, f: impl FnOnce(&StateLock<S>) -> R) -> R {
		self.with(move |s| {
			let was_dirty = s.meta.borrow().dirty;
			let res = f(s);
			if !was_dirty && s.meta.borrow().dirty {
				let _ = stdweb::web::window().request_animation_frame(move |_| crate::vdom::update(self));
			}
			res
		})
	}

	fn update<R>(&'static self, f: impl FnOnce(&mut S) -> R) -> R {
		self.with(move |s| {
			if !s.meta.borrow().dirty {
				let _ = stdweb::web::window().request_animation_frame(move |_| crate::vdom::update(self));
			}
			f(&mut s.update())
		})
	}
	fn view<R>(&'static self, f: impl FnOnce(&S) -> R) -> R {
		self.with(move |s| {
			f(&mut s.view())
		})
	}

	fn update_meta<R>(&'static self, f: impl FnOnce(&mut StateMeta) -> R) -> R {
		self.with(move |s| {
			if !s.meta.borrow().dirty {
				let _ = stdweb::web::window().request_animation_frame(move |_| crate::vdom::update(self));
			}
			f(&mut s.update_meta())
		})
	}
	fn view_meta<R>(&'static self, f: impl FnOnce(&StateMeta) -> R) -> R {
		self.with(move |s| {
			f(&mut s.view_meta())
		})
	}
}

impl<S: Default> StateLock<S> {
	pub fn update<'a>(&'a self) -> impl DerefMut<Target = S> + 'a {
		let mut meta = self.meta.borrow_mut();
		if !meta.dirty {
			meta.dirty = true;
		}
		self.state.borrow_mut()
	}

	pub fn update_meta<'a>(&'a self) -> impl DerefMut<Target = StateMeta> + 'a {
		let mut meta = self.meta.borrow_mut();
		if !meta.dirty {
			meta.dirty = true;
		}
		meta
	}

	pub fn view<'a>(&'a self) -> impl Deref<Target = S> + 'a {
		self.state.borrow()
	}

	pub fn view_meta<'a>(&'a self) -> impl Deref<Target = StateMeta> + 'a {
		self.meta.borrow()
	}
}
