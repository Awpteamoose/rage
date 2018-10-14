use crate::cmp::{StateRc, StateLock};
use std::{
	collections::{HashMap, hash_map::DefaultHasher},
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

pub fn styled(state_rc: &StateRc<impl Default>, css: &str) -> String {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);
	let _ = state_rc.borrow().styles.borrow_mut().insert(class.clone(), css.to_owned());

	class
}
