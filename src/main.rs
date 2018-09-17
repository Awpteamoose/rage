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
#![feature(try_from, try_trait, never_type, tool_lints)]
#![recursion_limit = "128"]

use std::{
	collections::HashMap,
	sync::{
		Mutex,
		Arc,
	},
};
use maplit::*;

#[derive(Default, Debug)]
struct State {
	some_value: i32,
}

#[derive(Default, Debug)]
struct StateLock {
	state: State,
	dirty: bool,
}

impl StateLock {
	fn update(&mut self, f: impl Fn(&mut State)) {
		f(&mut self.state);
		self.dirty = true;
	}
}

type Lock = Arc<Mutex<StateLock>>;

lazy_static::lazy_static! {
	static ref STATE_LOCK: Lock = Arc::new(Mutex::new(StateLock::default()));
}

#[derive(Debug)]
enum Prop {
	Number(i32),
	String(String),
}

#[derive(Debug)]
struct Div {
	state: Lock,
	children: Vec<Div>,
	props: HashMap<String, Prop>,
}

trait Component {
	fn render(&self) -> String;
}

fn styled(mut cmp: Div, css: String) -> Div {
	// TODO: use css
	cmp.props.insert(String::from("class"), Prop::String(String::from("poop")));
	cmp
}

impl Component for Div {
	fn render(&self) -> String {
		println!("{}", self.state.lock().unwrap().state.some_value);
		let children = self.children.iter().fold(String::new(), |acc, c| acc + &c.render());
		let props = self.props.iter().fold(String::new(), |acc, (name, value)| {
			acc + &format!("{}={}", name, &match value {
				Prop::Number(x) => x.to_string(),
				Prop::String(x) => format!(r#""{}""#, x),
			})
		});
		format!("<div {}>{}</div>", &props, &children)
	}
}

// fn styled(div

fn main() {
	let test_div1 = Div {
		state: Arc::clone(&STATE_LOCK),
		props: HashMap::new(),
		children: Vec::new(),
	};
	println!("{}", test_div1.render());
	{
		let mut state = STATE_LOCK.lock().unwrap();
		state.update(|s| s.some_value += 1);
	}
	let test_div2 = Div {
		state: Arc::clone(&STATE_LOCK),
		props: HashMap::new(),
		children: Vec::new(),
	};
	println!("{}", test_div2.render());
}
