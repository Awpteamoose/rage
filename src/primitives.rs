use crate::{StateRc, FnCmp};
use std::collections::HashMap;
use stdweb::{
	traits::*,
	web::{document, Node, Element},
};

macro_rules! primitive {
	($name: ident) => {
		#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
		pub fn $name(
			state_rc: &StateRc,
			children: &[FnCmp],
			attributes: &HashMap<String, String>,
			attach_events: impl Fn(&Element),
		) -> Node {
			let element = document().create_element(stringify!($name)).unwrap();

			for child in children {
				element.append_child(&child.0(&state_rc));
			}

			for (name, value) in attributes.iter() {
				element.set_attribute(name, value).unwrap();
			}

			attach_events(&element);

			element.into()
		}
	};
}

primitive!(div);

impl From<String> for FnCmp {
	fn from(s: String) -> Self {
		FnCmp(Box::new(move |_| document().create_text_node(&s).into()))
	}
}

impl From<&str> for FnCmp {
	fn from(s: &str) -> Self {
		let owned = s.to_owned();
		FnCmp(Box::new(move |_| document().create_text_node(&owned).into()))
	}
}
