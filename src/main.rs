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
#[macro_use]
mod primitives;
mod cmp;
mod styled;

use self::styled::styled;
use crate::cmp::*;
use futures::{join, try_join};
use maplit::*;
use std::{
	rc::Rc,
	collections::{
		HashMap,
		HashSet,
	},
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
	web::{error::Error, event, wait, Element},
	PromiseFuture,
};

// lazy_static::lazy_static! {
//     static ref STATE: StateRc<MyState> = StateRc::default();
// }

type Cells = HashSet<(u32, u32)>;

#[derive(Default, Debug)]
pub struct MyState {
	cells: Cells,
	some_value: i32,
	running: bool,
}

fn fetch(url: &str) -> PromiseFuture<String> {
	#[allow(clippy::result_unwrap_used)]
	js!(return fetch(@{url}).then((r)=>r.text());).try_into().unwrap()
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
	// spawn_local(unwrap_future(future_main()));

	let state_rc: StateRc<MyState> = StateRc::default();

	{
		let state_rc = Rc::clone(&state_rc);
		spawn_local(async move {
			loop {
				await!(wait(2000));
				if state_rc.borrow().state.running {
					console!(log, "tick");
				}
			}
		});
	}

	let rc1 = Rc::clone(&state_rc);
	let cells = move || {
		let mut divs = Vec::new();

		for x in 0..100 {
			for y in 0..100 {
				let state_rc = Rc::clone(&rc1);
				divs.push(primitives::div(
					children![],
					attrs![
						"class" => styled(&state_rc, &format!(r#"
							border: 1px solid black;
							background-color: {color};
							box-sizing: content-box;
						"#,
							color = if state_rc.borrow().state.cells.get(&(x, y)).is_some() { "black" } else { "white" }
						)),
					],
					|e| {
						let mut new_state = Rc::clone(&state_rc);
						let _ = e.add_event_listener(move |_: event::ClickEvent| {
							StateLock::update(&mut new_state, move |s| {
								// console!(log, format!("clicked ({}, {})", &x, &y));
								if s.cells.get(&(x, y)).is_some() { let _ = s.cells.remove(&(x, y)); }
								else { let _ = s.cells.insert((x, y)); }
							});
						});
					},
				));
			}
		}

		divs
	};

	let button = {
		let state_rc = Rc::clone(&state_rc);
		Cmp::new(move || -> Element { primitives::input(
			children![],
			attrs!["type" => "button".to_owned(), "value" => "go".to_owned()],
			|e| {
				let mut new_state = Rc::clone(&state_rc);
				let _ = e.add_event_listener(move |_: event::ClickEvent| {
					StateLock::update(&mut new_state, move |s| {
						console!(log, "clicked play stop");
						s.running = true;
					});
				});
			},
		)})
	};

	dom::mount(
		Rc::clone(&state_rc),
		Cmp::new(move || {

			let state = &state_rc.borrow().state;
			primitives::div(
				children![
					button.0().as_node(),
					primitives::div(&cells().iter().map(Element::as_node).collect::<Vec<_>>(), attrs![
						"class" => styled(&state_rc, r#"
							user-select: none;
							display: grid;
							grid-template-columns: repeat(100, 10px);
							grid-template-rows: repeat(100, 10px);
						"#),
					],
					|_| {}).as_node(),
				],
				attrs![],
				|e| {
					// let mut new_state = Rc::clone(&state_rc);
					// let _ = e.add_event_listener(move |_: event::ClickEvent| {
					//     console!(log, "clicky");
					//     StateLock::update(&mut new_state, move |s| {
					//         s.some_value += 1;
					//     });
					// });

					// let mut new_state = Rc::clone(&state_rc);
					// let _ = e.add_event_listener(move |e: event::AuxClickEvent| {
					//     if e.button() != event::MouseButton::Right {
					//         return;
					//     }
					//     console!(log, "rick clicky");
					//     StateLock::update(&mut new_state, move |s| {
					//         s.some_value -= 1;
					//     });
					// });
				},
			)
		}),
	);
}
