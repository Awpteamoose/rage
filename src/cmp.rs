use crate::dom;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use stdweb::web::{document, Element};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;
use std::ops::{Deref, DerefMut};
use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
};
use maplit::*;

#[allow(missing_debug_implementations)]
pub struct StateMeta<'vdom> {
	pub style: Element,
	pub mount: Box<dyn Fn() -> Element + Sync + Send>,
	pub styles: RwLock<HashMap<String, String>>,
	pub dirty: bool,
	pub vdom: crate::vdom::Element<'vdom>,
}

pub struct StateLockData<'vdom, S: Default> {
	pub meta: RwLock<StateMeta<'vdom>>,
	pub state: RwLock<S>,
}

pub struct StateLock<'vdom, S: Default>(pub StateLockData<'vdom, S>);

impl<'vdom, S: Default> Default for StateLock<'vdom, S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		StateLock(StateLockData {
			state: RwLock::new(S::default()),
			meta: RwLock::new(StateMeta {
				style: document().create_element("style").unwrap(),
				mount: Box::new(|| document().create_element("div").unwrap()),
				styles: RwLock::new(HashMap::default()),
				dirty: false,
				vdom: crate::vdom::Element {
					tag: crate::primitives::Tag::div,
					parent: None,
					children: vec![],
					attributes: hashmap![],
				},
			}),
		})
	}
}

impl<'vdom, S: Default + 'static> StateLock<'vdom, S> {
	pub fn update(&'static self) -> impl DerefMut<Target = S> + 'static {
		let mut meta = self.0.meta.write().unwrap();
		if !meta.dirty {
			meta.dirty = true;
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::dom::update(self));
		}
		self.0.state.write().unwrap()
		// let mut arc = Arc::clone(&self.state);
		// Arc::get_mut(&mut arc).unwrap()
	}

	pub fn update_meta(&'static self) -> impl DerefMut<Target = StateMeta<'vdom>> + 'static {
		// console!(log, self.meta.write().is_ok());
		let mut meta = self.0.meta.write().unwrap();
		if !meta.dirty {
			meta.dirty = true;
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::dom::update(self));
		}
		meta

		// let mut arc = Arc::clone(&self.meta);
		// Arc::get_mut(&mut arc).unwrap()
	}

	pub fn view(&'vdom self) -> impl Deref<Target = S> + 'vdom {
		self.0.state.read().expect("view panic")
		// Arc::clone(&self.state)
	}

	pub fn view_meta(&'vdom self) -> impl Deref<Target = StateMeta<'vdom>> + 'vdom {
		self.0.meta.read().expect("view meta panic")
		// Arc::clone(&self.meta)
	}
}
