// TODO: use fast hash
use std::{
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap, HashSet},
	hash::Hasher,
};
use stdweb::{
	traits::*,
	web::{document, Element as DomElement},
};

struct State {
	style_element: DomElement,
	inserted_rules: HashSet<String>,
}

thread_local! {
	static STATE: RefCell<State> = {
		let style_element = document().create_element("style").unwrap();
		document().head().unwrap().append_child(&style_element);
		RefCell::new(State {
			style_element,
			inserted_rules: HashSet::new(),
		})
	};
}

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

#[allow(unused_must_use)]
pub fn styled(css: &str) -> String {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);

	STATE.with(|s| {
		let mut state = s.borrow_mut();
		if !state.inserted_rules.contains(&class) {
			let style = format!(".{} {{ {} }}\n", &class, css);
			let _ = state.inserted_rules.insert(class.clone());
			state.style_element.append_html(&style);
		}
	});

	class
}

pub fn append_css(css: &str) {
	STATE.with(|state| state.borrow().style_element.append_html(&style));
}
