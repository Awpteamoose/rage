use crate::cmp::StateRc;
use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
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
