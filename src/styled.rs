use crate::{FnCmp, State, StateRc};
use std::{
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
};
use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
	traits::*,
	unstable::TryFrom,
	web::{Element, Node},
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub fn styled(state_rc: &StateRc, element: Element, css: &str) -> Element {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);
	let _ = element.set_attribute("class", &class);
	let _ = state_rc.borrow().styles.borrow_mut().insert(class, css.to_owned());

	element
}
