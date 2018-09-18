use std::{
	collections::{
		HashMap,
		hash_map::DefaultHasher,
	},
	hash::Hasher,
};
use stdweb::web::Node;
use crate::{
	Component,
	StateRc,
	State,
};

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

pub struct Styled<CMP: Component, F: Fn(&CMP, &State) -> String> {
	pub inner: CMP,
	pub get_css: F,
}

impl<CMP: Component, F: Fn(&CMP, &State) -> String> Component for Styled<CMP, F> {
	fn render(&mut self, state: StateRc) -> Node {
		{
			let css = (self.get_css)(&self.inner, &state.borrow().state);
			let class = hash(&css);
			let _ = self.attributes().insert(String::from("class"), class.clone());
			let _ = state.borrow().styles.borrow_mut().insert(class, css);
		}
		self.inner.render(state)
	}
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { self.inner.children() }
	fn attributes(&mut self) -> &mut HashMap<String, String> { self.inner.attributes() }
}
