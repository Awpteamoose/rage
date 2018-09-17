#![deny(rust_2018_idioms)]
#![warn(
	clippy::clippy,
	clippy::clippy_pedantic,
	clippy::clippy_style,
	clippy::clippy_complexity,
	clippy::clippy_perf,
	clippy::clippy_correctness,

	// specific lints
	// Restriction (all of Restrictio is Allow)
	clippy::clone_on_ref_ptr,
	clippy::float_cmp_const,
	clippy::option_unwrap_used,
	clippy::result_unwrap_used,
	clippy::wrong_pub_self_convention,
	clippy::shadow_unrelated,

	// Pedantic (these are Allow in Pedantic)
	clippy::empty_enum,
	clippy::enum_glob_use,
	clippy::if_not_else,
	clippy::items_after_statements,
	clippy::mut_mut,
	clippy::needless_continue,
	clippy::pub_enum_variant_names,
	clippy::replace_consts,
	clippy::result_map_unwrap_or_else,
	clippy::stutter,
	clippy::use_self,

	// default rust lints that are Allow
	// https://doc.rust-lang.org/nightly/rustc/lints/listing/allowed-by-default.html
	anonymous_parameters,
	bare_trait_objects,
	missing_debug_implementations,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_extern_crates,
	unused_import_braces,
	unused_qualifications,
	unused_results
)]
#![cfg_attr(not(debug_assertions), warn(
	// Restriction
	clippy::use_debug,
	clippy::print_stdout,
	clippy::unimplemented,
))]
#![allow(clippy::result_unwrap_used, clippy::option_unwrap_used)]
#![feature(try_from, try_trait, never_type, tool_lints, set_stdio)]
#![recursion_limit = "128"]

use std::{
	collections::{
		HashMap,
		hash_map::DefaultHasher,
	},
	hash::Hasher,
	cell::RefCell,
	rc::Rc,
};
use stdweb::{
	// js, _js_impl, __js_raw_asm,
	// console, __internal_console_unsafe,
	web::{
		document,
		HtmlElement,
		event,
		Node,
	},
	unstable::TryFrom,
	traits::*,
};
// use maplit::*;

#[derive(Default, Debug)]
struct State {
	some_value: i32,
}

struct StateLock {
	root: HtmlElement,
	style: HtmlElement,
	mount: Rc<RefCell<Vec<Box<dyn Component>>>>,
	styles: Rc<RefCell<HashMap<String, String>>>,
	state: State,
}

impl Default for StateLock {
	fn default() -> Self {
		Self {
			root: HtmlElement::try_from(document().create_element("div").unwrap()).unwrap(),
			style: HtmlElement::try_from(document().create_element("style").unwrap()).unwrap(),
			mount: Rc::new(RefCell::new(Vec::new())),
			styles: Rc::new(RefCell::new(HashMap::new())),
			state: State::default(),
		}
	}
}

impl StateLock {
	fn update(arc: &mut StateRc, f: impl Fn(&mut State)) {
		f(&mut arc.borrow_mut().state);
		update_dom(arc);
	}
}

type StateRc = Rc<RefCell<StateLock>>;

struct Div {
	children: Vec<Box<dyn Component>>,
	attributes: HashMap<String, String>,
}

trait Component: Send + Sync {
	fn render(&mut self, _: StateRc) -> Node;
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { unimplemented!() }
	fn attributes(&mut self) -> &mut HashMap<String, String> { unimplemented!() }
}

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

struct Styled<CMP: Component, F: Sync + Send + Fn(&CMP, &State) -> String> {
	inner: CMP,
	get_css: F,
}

impl<CMP: Component, F: Sync + Send + Fn(&CMP, &State) -> String> Component for Styled<CMP, F> {
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

impl Component for Div {
	fn render(&mut self, mut state: StateRc) -> Node {
		let div = document().create_element("div").unwrap();

		for child in self.children.iter_mut() {
			div.append_child(&child.render(Rc::clone(&state)));
		}

		for (name, value) in self.attributes.iter() {
			div.set_attribute(name, value).unwrap();
		}

		let _handler = div.add_event_listener(move |_e: event::ClickEvent| {
			StateLock::update(&mut state, |s| {
				s.some_value += 1;
			});
		});

		div.into()
	}
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { &mut self.children }
	fn attributes(&mut self) -> &mut HashMap<String, String> { &mut self.attributes }
}

impl Component for String {
	fn render(&mut self, _state: StateRc) -> Node {
		document().create_text_node(&self).into()
	}
}

impl Component for &str {
	fn render(&mut self, _state: StateRc) -> Node {
		document().create_text_node(&self.to_string()).into()
	}
}

impl<T: ToString + Send + Sync, F: Fn(StateRc) -> T + Send + Sync> Component for F {
	fn render(&mut self, state: StateRc) -> Node {
		document().create_text_node(&self(state).to_string()).into()
	}
}

fn update_dom(state: &StateRc) {
	let mut nodes = Vec::new();
	for cmp in state.borrow().mount.borrow_mut().iter_mut() {
		nodes.push(cmp.render(Rc::clone(&state)));
	}
	let StateLock { style, root, styles, .. } = &mut state.borrow_mut() as &mut StateLock;

	styles.borrow_mut().clear();
	root.set_text_content("");

	style.set_text_content(&styles.borrow_mut().iter().fold(String::new(), |acc, (class, style)| {
		acc + &format!(".{} {{ {} }}", class, style)
	}));

	for node in nodes {
		root.append_child(&node);
	}
}

fn main() {
	let state: StateRc = StateRc::default();
	{
		let state_write = &mut state.borrow_mut() as &mut StateLock;

		let test_div2 = Styled {
			inner: Div {
				attributes: HashMap::new(),
				children: vec![Box::new("pidoir")],
			},
			get_css: |_, state: &State| {
				format!("width: {}px", state.some_value)
			},
		};

		let test_div3 = Div {
			attributes: HashMap::new(),
			children: vec![Box::new(test_div2)],
		};

		let test_div = Div {
			attributes: HashMap::new(),
			children: vec![Box::new(test_div3), Box::new(|state: StateRc| format!("more pidoir {}", state.borrow().state.some_value))],
		};

		state_write.mount.borrow_mut().push(Box::new(test_div));

		document().head().unwrap().append_child(&state_write.style);
		document().body().unwrap().append_child(&state_write.root);
	}

	update_dom(&state);
}
