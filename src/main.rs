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
#![feature(async_await, await_macro, futures_api, pin)]

mod dom;
#[macro_use] mod primitives;
mod styled;

use self::{primitives::*, styled::styled};
use futures::{join, try_join};
use maplit::*;
use std::{
	any::Any,
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
	rc::Rc,
};
use stdweb::{
	__internal_console_unsafe,
	__js_raw_asm,
	_js_impl,
	console,
	js,
	spawn_local,
	traits::*,
	unstable::{TryFrom, TryInto},
	unwrap_future,
	web::{document, error::Error, event, wait, Element, HtmlElement, Node},
	PromiseFuture,
};

#[derive(Default, Debug)]
pub struct State {
	some_value: i32,
}

pub struct StateLock {
	pub style: Element,
	pub mount: FnCmp,
	pub styles: Rc<RefCell<HashMap<String, String>>>,
	pub state: State,
}

impl Default for StateLock {
	fn default() -> Self {
		Self {
			style: document().create_element("style").unwrap(),
			mount: FnCmp(Box::new(|_| document().create_element("div").unwrap())),
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

pub struct FnCmp(Box<dyn Fn(&StateRc) -> Element>);

// Converts a JavaScript Promise into a Rust Future
fn javascript_promise() -> PromiseFuture<u32> {
	js!(
		return new Promise( function ( success, error ) {
			setTimeout( function () {
				success( 50 );
			}, 2000 );
		} );
	)
	.try_into()
	.unwrap()
}

async fn print(message: &str) {
	// Waits for 2000 milliseconds
	await!(wait(2000));
	console!(log, message);
}

async fn future_main() -> Result<(), Error> {
	// Runs Futures synchronously
	await!(print("Hello"));
	await!(print("There"));

	{
		let a = print("Test 1");
		let b = print("Test 2");

		// Runs multiple Futures in parallel
		join!(a, b);

		console!(log, "Done");
	}

	{
		let a = javascript_promise();
		let b = javascript_promise();

		// Runs multiple Futures (which can error) in parallel
		let (a, b) = try_join!(a, b)?;

		console!(log, a, b);
	}

	Ok(())
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	spawn_local(unwrap_future(future_main()));

	let state_rc: StateRc = StateRc::default();
	{
		let state_lock: &mut StateLock = &mut state_rc.borrow_mut();

		let test_div = FnCmp(Box::new(|state_rc: &StateRc| {
			let state = &state_rc.borrow().state;

			styled(
				&state_rc,
				div(
					&state_rc,
					children!["Shitty ", format!("more {}", state.some_value)],
					attrs![],
					|e| {
						let mut new_state = Rc::clone(&state_rc);
						let _ = e.add_event_listener(move |_: event::ClickEvent| {
							console!(log, "clicky");
							StateLock::update(&mut new_state, move |s| {
								s.some_value += 1;
							});
						});

						let mut new_state = Rc::clone(&state_rc);
						let _ = e.add_event_listener(move |e: event::AuxClickEvent| {
							if e.button() != event::MouseButton::Right {
								return;
							}
							console!(log, "rick clicky");
							StateLock::update(&mut new_state, move |s| {
								s.some_value -= 1;
							});
						});
					},
				),
				&format!(r#"
					font-size: {size}px;
					user-select: none;
				"#,
					size = (state.some_value + 5) * 10
				),
			)
		}));

		let _ = std::mem::replace(&mut state_lock.mount, test_div);

		document().head().unwrap().append_child(&state_lock.style);
	}

	dom::update(&state_rc);
}
