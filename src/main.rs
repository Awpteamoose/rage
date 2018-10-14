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

use self::{primitives::*, styled::styled};
use std::{
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
	rc::Rc,
	any::Any,
};
use stdweb::{
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	traits::*,
	unstable::TryFrom,
	web::{document, event, HtmlElement, Node, Element},
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
			mount: FnCmp(Box::new(|_| Node::from(document().create_element("div").unwrap()))),
			styles: Rc::new(RefCell::new(HashMap::new())),
			state: State::default(),
		}
	}
}

impl StateLock {
	fn update(state_rc: &mut StateRc, f: impl Fn(&mut State)) {
		f(&mut state_rc.borrow_mut().state);
		dom::update(state_rc);
	}
}

pub type StateRc = Rc<RefCell<StateLock>>;

pub struct FnCmp(Box<dyn Fn (StateRc) -> Node>);

fn main() {
	let state_rc: StateRc = StateRc::default();
	{
		let state_lock: &mut StateLock = &mut state_rc.borrow_mut();

		macro_rules! children {
			($($e: expr),+$(,)*) => {
				[$($e.into(),)+]
			};
		};

		let test_div = FnCmp(Box::new(|state_rc: StateRc| {
			let state = &state_rc.borrow().state;

			styled(div, format!(r#"font-size: {}px;"#, (state.some_value + 5) * 10))(
				&state_rc,
				&children![
					"Shitty\n",
					format!("more {}", state.some_value),
				],
				&hashmap![],
				|e| {
					let mut new_state = Rc::clone(&state_rc);
					let _ = e.add_event_listener(move |_: event::ClickEvent| {
						console!(log, "clicky");
						StateLock::update(&mut new_state, move |s| {
							s.some_value += 1;
						});
					});

					// let mut new_state = Rc::clone(&state_rc);
					// let _ = e.add_event_listener(move |_: event::MouseEnterEvent| {
					//     console!(log, "mouse enter");
					//     StateLock::update(&mut new_state, move |s| {
					//         s.some_value -= 1;
					//     });
					// });
				},
			)
		}));

		let _ = std::mem::replace(&mut state_lock.mount, test_div);

		document().head().expect("no head").append_child(&state_lock.style);
	}

	dom::update(&state_rc);
}
