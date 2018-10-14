use crate::{State, StateRc, FnCmp};
use std::{
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
};
use stdweb::web::{Node, Element};
use stdweb::{
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub fn styled<
	F: Fn(&Element),
	P: Fn(
		&StateRc,
		&[FnCmp],
		&HashMap<String, String>,
		F,
	) -> Node,
>(f: P, css: String) -> impl Fn(
		&StateRc,
		&[FnCmp],
		&HashMap<String, String>,
		F,
) -> Node {
	console!(log, &css);
	move |state_rc: &StateRc, children: &[FnCmp], attributes: &HashMap<String, String>, attach_events: F| -> Node {
		let attrs = {
			let class_hash = hash(&css);
			let mut new_attributes = attributes.clone();
			let class = format!("styled{}", &class_hash);
			let _ = new_attributes.insert(String::from("class"), class.clone());
			let _ = state_rc.borrow().styles.borrow_mut().insert(class, css.clone());
			new_attributes
		};
		console!(log, format!("{:?}", attrs));
		f(state_rc, children, &attrs, attach_events)
	}
}
