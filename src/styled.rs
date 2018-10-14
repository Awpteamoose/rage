use crate::cmp::{StateRc, StateLock};
use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
};
use stdweb::{
	traits::*,
	web::Element,
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub fn styled(state_lock: &StateLock<impl Default>, element: Element, css: &str) -> Element {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);
	let _ = element.set_attribute("class", &class);
	let _ = state_lock.styles.borrow_mut().insert(class, css.to_owned());

	element
}
