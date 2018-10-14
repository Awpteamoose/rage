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
mod cmp;

use self::styled::styled;
use futures::{join, try_join};
use maplit::*;
use std::{
	cell::RefCell,
	collections::HashMap,
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
	unstable::TryInto,
	unwrap_future,
	web::{document, error::Error, event, wait, Element},
	PromiseFuture,
};
use crate::cmp::*;

#[derive(Default, Debug)]
pub struct MyState {
	some_value: i32,
}

fn fetch(url: &str) -> PromiseFuture<String> {
	#[allow(clippy::result_unwrap_used)]
	js!(return fetch(@{url}).then((r)=>r.text());)
		.try_into()
		.unwrap()
}

async fn print(message: &str) {
	// Waits for 2000 milliseconds
	await!(wait(2000));
	console!(log, message);
}

#[allow(clippy::useless_let_if_seq)]
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
		let a = fetch("https://logcraft.grdigital.co.uk/version");
		let b = fetch("https://logcraft.grdigital.co.uk/version");

		// Runs multiple Futures (which can error) in parallel
		let (a, b) = try_join!(a, b)?;

		console!(log, a, b);
	}

	Ok(())
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	spawn_local(unwrap_future(future_main()));

	let state_rc: StateRc<MyState> = StateRc::default();

	let mount = {
		// let state_lock: &mut StateLock<MyState> = &mut state_rc.borrow_mut();

		let state_rc = Rc::clone(&state_rc);
		FnCmp::new(move |state_lock: &StateLock<MyState>| {
			let state_rc = Rc::clone(&state_rc);
			styled(
				state_lock,
				primitives::div(
					state_lock,
					children!["Shitty ", format!("more {}", state_lock.state.some_value)],
					attrs![],
					move |e| {
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
					size = (state_lock.state.some_value + 5) * 10
				),
			)
		})
	};

	let _ = std::mem::replace(&mut state_rc.borrow_mut().mount, mount);

	document().head().unwrap().append_child(&state_rc.borrow().style);

	dom::update(&state_rc);
}
