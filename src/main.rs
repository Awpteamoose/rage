#![deny(
	rust_2018_idioms,
	unused_must_use, // warn by default
)]
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
	clippy::shadow_reuse,

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
	clippy::shadow_unrelated,

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
	unused_results,
)]
#![cfg_attr(not(debug_assertions), warn(
	// Restriction
	clippy::use_debug,
	clippy::print_stdout,
	clippy::unimplemented,
))]
#![feature(tool_lints)]
#![recursion_limit = "128"]

#![allow(unreachable_pub)]
#![feature(try_from, try_trait, never_type)]

mod primitives;
mod styled;
mod dom;

use self::{primitives::*, styled::*, dom::*};
use std::{
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
	rc::Rc,
};
use stdweb::{
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	traits::*,
	unstable::TryFrom,
	web::{document, event, HtmlElement, Node},
};
use maplit::*;

#[derive(Default, Debug)]
pub struct State {
	some_value: i32,
}

pub struct StateLock {
	pub style: HtmlElement,
	pub mount: FnCmp,
	pub styles: Rc<RefCell<HashMap<String, String>>>,
	pub state: State,
}

impl Default for StateLock {
	fn default() -> Self {
		Self {
			style: HtmlElement::try_from(document().create_element("style").unwrap()).unwrap(),
			mount: FnCmp(Box::new(|| Node::from(document().create_element("div").unwrap()))),
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

pub type StateRc = Rc<RefCell<StateLock>>;

pub trait Component {
	fn render(&mut self, _: StateRc) -> Node;
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { unimplemented!() }
	fn attributes(&mut self) -> &mut HashMap<String, String> { unimplemented!() }
}

pub struct FnCmp(Box<dyn Fn () -> Node>);

fn div_f(children: &Vec<FnCmp>, attributes: HashMap<String, String>) -> Node {
	let element = document().create_element("div").unwrap();

	for child in children {
		element.append_child(&child.0());
	}

	for (name, value) in attributes.iter() {
		element.set_attribute(name, value).unwrap();
	}

	element.into()
}

fn main() {
	let state_rc: StateRc = StateRc::default();
	{
		let state_lock: &mut StateLock = &mut state_rc.borrow_mut();
		let mut new_state = Rc::clone(&state_rc);

		// macro_rules! children {
		//     ($($e: expr),+$(,)*) => {
		//         vec![$(Box::new($e),)+]
		//     };
		// };

		// let test_div = Div::new()
		//     .children(children![
		//         Div::new().children(children!["div1"]),
		//         Div::new().children(children!["div2"]),
		//         Div::new()
		//             .children(children![
		//                 |state: &State| format!("more {}", state.some_value),
		//             ])
		//             .on_click(move |_| {
		//                 StateLock::update(&mut new_state, move |s| {
		//                     s.some_value += 1;
		//                 });
		//             }),
		//     ]);

		macro_rules! children {
			($($e: expr),+$(,)*) => {
				vec![$($e.into(),)+]
			};
		};
		let test_div = FnCmp(Box::new(|| {
			div_f(&children!["Shitty"], hashmap![])
		}));

		std::mem::replace(&mut state_lock.mount, test_div);

		document().head().expect("no head").append_child(&state_lock.style);
	}

	update_dom(&state_rc);
}
