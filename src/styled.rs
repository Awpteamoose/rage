use crate::{State, StateRc, FnCmp};
use std::{
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
};
use stdweb::web::{Node, Element};
use stdweb::{
	traits::*,
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	unstable::TryFrom,
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
