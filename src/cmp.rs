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

#[allow(missing_debug_implementations)]
pub struct StateMeta {
	pub style: Element,
	pub mount: Box<dyn Fn() -> Element + Sync + Send>,
	pub styles: RwLock<HashMap<String, String>>,
	pub dirty: bool,
}

pub struct StateLockData<S: Default> {
	pub meta: RwLock<StateMeta>,
	pub state: RwLock<S>,
}

pub struct StateLock<S: Default>(pub Arc<StateLockData<S>>);

impl<S: Default> Default for StateLock<S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		StateLock(Arc::new(StateLockData {
			state: RwLock::new(S::default()),
			meta: RwLock::new(StateMeta {
				style: document().create_element("style").unwrap(),
				mount: Box::new(|| document().create_element("div").unwrap()),
				styles: RwLock::new(HashMap::default()),
				dirty: false,
			}),
		}))
	}
}

impl<S: Default + 'static> StateLock<S> {
	pub fn update<'a>(&'a self) -> impl DerefMut<Target = S> + 'a {
		let mut meta = self.0.meta.write().unwrap();
		if !meta.dirty {
			meta.dirty = true;
			let arc = Arc::clone(&self.0);
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::dom::update(&StateLock(arc)));
		}
		self.0.state.write().unwrap()
		// let mut arc = Arc::clone(&self.state);
		// Arc::get_mut(&mut arc).unwrap()
	}

	pub fn update_meta<'a>(&'a self) -> impl DerefMut<Target = StateMeta> + 'a {
		// console!(log, self.meta.write().is_ok());
		let mut meta = self.0.meta.write().unwrap();
		if !meta.dirty {
			meta.dirty = true;
			let arc = Arc::clone(&self.0);
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::dom::update(&StateLock(arc)));
		}
		meta

		// let mut arc = Arc::clone(&self.meta);
		// Arc::get_mut(&mut arc).unwrap()
	}

	pub fn view<'a>(&'a self) -> impl Deref<Target = S> + 'a {
		self.0.state.read().expect("view panic")
		// Arc::clone(&self.state)
	}

	pub fn view_meta<'a>(&'a self) -> impl Deref<Target = StateMeta> + 'a {
		self.0.meta.read().expect("view meta panic")
		// Arc::clone(&self.meta)
	}
}
