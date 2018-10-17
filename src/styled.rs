use crate::cmp::StateLock;
use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub fn styled(lock: &'static StateLock<impl Default + 'static>, css: &str) -> String {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);
	let _ = lock.view_meta().styles.write().unwrap().insert(class.clone(), css.to_owned());

	class
}
