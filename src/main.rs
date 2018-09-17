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
#![feature(try_from, try_trait, never_type, tool_lints)]
#![recursion_limit = "128"]

use std::{
	collections::{
		HashMap,
		hash_map::DefaultHasher,
	},
	hash::Hasher,
	sync::{
		Mutex,
		Arc,
	},
};
// use maplit::*;

#[derive(Default, Debug)]
struct State {
	some_value: i32,
}

#[derive(Default, Debug)]
struct StateLock {
	styles: HashMap<String, String>,
	state: State,
	dirty: bool,
}

impl StateLock {
	fn update(&mut self, f: impl Fn(&mut State)) {
		f(&mut self.state);
		self.dirty = true;
	}
}

type StateArc = Arc<Mutex<StateLock>>;

lazy_static::lazy_static! {
	static ref STATE: StateArc = Arc::new(Mutex::new(StateLock::default()));
}

#[derive(Debug)]
enum Prop {
	Number(i32),
	String(String),
}

#[derive(Debug)]
struct Div {
	children: Vec<Box<dyn Component>>,
	props: HashMap<String, Prop>,
}

trait Component: Send + Sync + std::fmt::Debug {
	fn render(&mut self) -> String;
	fn children(&mut self) -> &mut Vec<Box<dyn Component>>;
	fn props(&mut self) -> &mut HashMap<String, Prop>;
}

fn hash(s: &str) -> String {
	let mut hasher = DefaultHasher::new();
	hasher.write(s.as_bytes());
	hasher.finish().to_string()
}

struct Styled<CMP: Component, F: Sync + Send + Fn(&CMP) -> String> {
	inner: CMP,
	get_css: F,
}

impl<CMP: Component, F: Sync + Send + Fn(&CMP) -> String> std::fmt::Debug for Styled<CMP, F> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.inner.fmt(f) }
}

impl<CMP: Component, F: Sync + Send + Fn(&CMP) -> String> Component for Styled<CMP, F> {
	fn render(&mut self) -> String {
		let css = (self.get_css)(&self.inner);
		let class = hash(&css);
		let _ = self.props().insert(String::from("class"), Prop::String(class.clone()));
		let _ = STATE.lock().unwrap().styles.insert(class, css);
		self.inner.render()
	}
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { self.inner.children() }
	fn props(&mut self) -> &mut HashMap<String, Prop> { self.inner.props() }
}

fn styled<CMP: Component, F: Sync + Send + Fn(&CMP) -> String>(cmp: CMP, get_css: F) -> Styled<CMP, F> {
	Styled { inner: cmp, get_css }
}

impl Component for Div {
	fn render(&mut self) -> String {
		STATE.lock().unwrap().update(|s| s.some_value += 1);
		let children = self.children.iter_mut().fold(String::new(), |acc, c| acc + &c.render());
		let props = self.props.iter().fold(String::new(), |acc, (name, value)| {
			acc + &format!("{}={}", name, &match value {
				Prop::Number(x) => x.to_string(),
				Prop::String(x) => format!(r#""{}""#, x),
			})
		});
		format!("<div {}>{}</div>", &props, &children)
	}
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { &mut self.children }
	fn props(&mut self) -> &mut HashMap<String, Prop> { &mut self.props }
}

impl Component for String {
	fn render(&mut self) -> String { self.clone() }
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { unimplemented!() }
	fn props(&mut self) -> &mut HashMap<String, Prop> { unimplemented!() }
}

impl Component for &str {
	fn render(&mut self) -> String { String::from(*self) }
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { unimplemented!() }
	fn props(&mut self) -> &mut HashMap<String, Prop> { unimplemented!() }
}

fn main() {
	let mut test_div1 = Div {
		props: HashMap::new(),
		children: Vec::new(),
	};
	println!("{}", test_div1.render());

	let mut test_div2 = styled(Div {
		props: HashMap::new(),
		children: vec![Box::new("inner pidor")],
	}, |_| format!("width: {}px", STATE.lock().unwrap().state.some_value));
	println!("{}", test_div2.render());

	let mut test_div3 = Div {
		props: HashMap::new(),
		children: vec![Box::new(test_div2)],
	};
	println!("{}", test_div3.render());

	println!("{:?}", *STATE.lock().unwrap());
}
