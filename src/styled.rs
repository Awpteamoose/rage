use crate::StateRc;
use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
};
use stdweb::{
	traits::*,
	web::Element,
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub fn styled(state_rc: &StateRc<impl Default>, element: Element, css: &str) -> Element {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);
	let _ = element.set_attribute("class", &class);
	let _ = state_rc.borrow().styles.borrow_mut().insert(class, css.to_owned());

	element
}
