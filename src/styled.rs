use std::{
	collections::hash_map::DefaultHasher,
	hash::Hasher,
	cell::RefCell,
	collections::{HashMap, HashSet},
};
use stdweb::{traits::*, web::{document, Element as DomElement}};

struct State {
	dirty: bool,
	style_element: DomElement,
	inserted_rules: HashSet<String>,
	new_rules: HashMap<String, String>,
}

thread_local! {
	static STATE: RefCell<State> = {
		let style_element = document().create_element("style").unwrap();
		document().head().unwrap().append_child(&style_element);
		RefCell::new(State {
			dirty: false,
			style_element,
			inserted_rules: HashSet::new(),
			new_rules: HashMap::new(),
		})
	};
}

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

fn insert_new_rules(_: f64) {
	STATE.with(|s| {
		let mut state = s.borrow_mut();
		let new_rules = std::mem::replace(&mut state.new_rules, HashMap::new());
		let new_css = new_rules.into_iter().fold(String::new(), |acc, (class, css)| {
			let style = format!(".{} {{ {} }}", &class, css);
			let _ = state.inserted_rules.insert(class);
			acc + &style
		});
		state.style_element.append_html(&new_css).unwrap();
	});
}

pub fn styled(css: &str) -> String {
	let class_hash = hash(&css);
	let class = format!("styled{}", &class_hash);

	STATE.with(|s| {
		let mut state = s.borrow_mut();
		if !state.inserted_rules.contains(&class) {
			let _ = state.new_rules.insert(class.clone(), css.to_owned());
			if !state.dirty {
				state.dirty = true;
				let _ = stdweb::web::window().request_animation_frame(insert_new_rules);
			}
		}
	});

	class
}
