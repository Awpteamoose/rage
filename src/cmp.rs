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
pub struct StateMeta {
	pub style: Element,
	pub mount: Box<dyn Fn() -> crate::vdom::Element + Sync + Send>,
	pub styles: RwLock<HashMap<String, String>>,
	pub dirty: bool,
	pub vdom: crate::vdom::Element,
}

pub struct StateLockData<S: Default> {
	pub meta: RwLock<StateMeta>,
	pub state: RwLock<S>,
}

pub struct StateLock<S: Default>(pub StateLockData<S>);

impl<S: Default> Default for StateLock<S> {
	#[allow(clippy::result_unwrap_used)]
	fn default() -> Self {
		StateLock(StateLockData {
			state: RwLock::new(S::default()),
			meta: RwLock::new(StateMeta {
				style: document().create_element("style").expect(&format!("{}:{}", file!(), line!())),
				mount: Box::new(|| {
					console!(log, "NIGGER NIGGER NIGGER");
					unimplemented!()
				}),
				styles: RwLock::new(HashMap::default()),
				dirty: false,
				vdom: crate::vdom::Element::new(
					crate::primitives::Tag::div,
					vec![],
					attrs!["nigg" => "lol"],
					vec![],
				),
			}),
		})
	}
}

impl<S: Default + 'static> StateLock<S> {
	pub fn update(&'static self) -> impl DerefMut<Target = S> + 'static {
		let mut meta = self.0.meta.write().expect(&format!("{}:{}", file!(), line!()));
		if !meta.dirty {
			meta.dirty = true;
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::vdom::update(self));
		}
		self.0.state.write().expect(&format!("{}:{}", file!(), line!()))
		// let mut arc = Arc::clone(&self.state);
		// Arc::get_mut(&mut arc).expect(&format!("{}:{}", file!(), line!()))
	}

	pub fn update_meta(&'static self) -> impl DerefMut<Target = StateMeta> + 'static {
		// console!(log, self.meta.write().is_ok());
		let mut meta = self.0.meta.write().expect(&format!("{}:{}", file!(), line!()));
		if !meta.dirty {
			meta.dirty = true;
			let _ = stdweb::web::window().request_animation_frame(move |_| crate::vdom::update(self));
		}
		meta

		// let mut arc = Arc::clone(&self.meta);
		// Arc::get_mut(&mut arc).expect(&format!("{}:{}", file!(), line!()))
	}

	pub fn view(&'static self) -> impl Deref<Target = S> {
		self.0.state.read().expect("view panic")
		// Arc::clone(&self.state)
	}

	pub fn view_meta(&'static self) -> impl Deref<Target = StateMeta> {
		self.0.meta.read().expect("view meta panic")
		// Arc::clone(&self.meta)
	}
}
