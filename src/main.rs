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

const GRID_SIZE: u32 = 100;
const CELL_SIZE: u32 = 10;
type Cell = (u32, u32);
type Cells = HashSet<(u32, u32)>;

fn neighbours(cells: &Cells, point: Cell) -> Vec<Cell> {
	let mut res = Vec::new();

	// SW
	if let (Some(x), Some(y)) = (point.0.checked_sub(1), point.1.checked_sub(1)) {
		if cells.get(&(x, y)).is_some() { res.push((x, y)); }
	}
	// W
	if let (Some(x), y) = (point.0.checked_sub(1), point.1) {
		if cells.get(&(x, y)).is_some() { res.push((x, y)); }
	}
	// S
	if let (x, Some(y)) = (point.0, point.1.checked_sub(1)) {
		if cells.get(&(x, y)).is_some() { res.push((x, y)); }
	}
	// SE
	if let (x, Some(y)) = (point.0 + 1, point.1.checked_sub(1)) {
		if cells.get(&(x, y)).is_some() { res.push((x, y)); }
	}
	// NW
	if let (Some(x), y) = (point.0.checked_sub(1), point.1 + 1) {
		if cells.get(&(x, y)).is_some() { res.push((x, y)); }
	}
	// E
	if cells.get(&(point.0 + 1, point.1)).is_some() { res.push((point.0, point.1)); }
	// N
	if cells.get(&(point.0, point.1 + 1)).is_some() { res.push((point.0, point.1)); }
	// NE
	if cells.get(&(point.0 + 1, point.1 + 1)).is_some() { res.push((point.0, point.1)); }

	res
}

lazy_static::lazy_static! {
	static ref RNG: std::sync::Mutex<rand::rngs::SmallRng> = {
		use rand::prelude::*;

		let mut bytes: [u8; 16] = [0; 16];
		let seed: [u8; 8] = unsafe { std::mem::transmute(stdweb::web::Date::new().get_time()) };
		for (index, byte) in seed.iter().enumerate() {
			bytes[index] = *byte;
			bytes[index + 8] = *byte;
		}
		std::sync::Mutex::new(rand::rngs::SmallRng::from_seed(bytes))
	};
}

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

fn cells(rc: &StateRc<MyState>) -> Vec<Element> {
	let mut divs = Vec::new();

	for x in 0..GRID_SIZE {
		for y in 0..GRID_SIZE {
			divs.push(primitives::div(
				children![],
				attrs![
					"class" => styled(&rc, &format!(r#"
						border: 1px solid black;
						background-color: {color};
						box-sizing: content-box;
					"#,
						color = if rc.borrow().state.cells.get(&(x, y)).is_some() { "black" } else { "white" }
					)),
				],
				|e| {
					let mut new_state = Rc::clone(&rc);
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
}

fn start_button(rc: &StateRc<MyState>) -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => if rc.borrow().state.running { "stop" } else { "start" },
		],
		|e| {
			let mut new_state = Rc::clone(&rc);
			let _ = e.add_event_listener(move |_: event::ClickEvent| {
				StateLock::update(&mut new_state, move |s| {
					s.running = !s.running;
				});
			});
		},
	)
}

fn randomize_button(rc: &StateRc<MyState>) -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => "randomize",
		],
		|e| {
			let mut new_state = Rc::clone(&rc);
			let _ = e.add_event_listener(move |_: event::ClickEvent| {
				StateLock::update(&mut new_state, move |s| {
					use rand::prelude::*;

					for x in 0..GRID_SIZE {
						for y in 0..GRID_SIZE {
							if RNG.lock().unwrap().next_u32() > (u32::max_value() / 2) {
								let _ = s.cells.insert((x, y));
							} else {
								let _ = s.cells.remove(&(x, y));
							}
						}
					}
				});
			});
		},
	)
}

fn container(rc: &StateRc<MyState>) -> Element {
	primitives::div(
		&cells(rc).iter().map(Element::as_node).collect::<Vec<_>>(),
		attrs![
			"class" => styled(rc, &format!(r#"
				user-select: none;
				display: grid;
				grid-template-columns: repeat({grid_size}, {cell_size}px);
				grid-template-rows: repeat({grid_size}, {cell_size}px);
			"#, grid_size = GRID_SIZE, cell_size = CELL_SIZE)),
		],
		|_| {}
	)
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	// spawn_local(unwrap_future(future_main()));

	let state_rc: StateRc<MyState> = StateRc::default();

	{
		let mut state_rc = Rc::clone(&state_rc);
		spawn_local(async move {
			loop {
				await!(wait(250));
				StateLock::update(&mut state_rc, |state| {
					let mut living = Vec::new();
					let mut dead = Vec::new();

					if state.running {
						console!(log, "TICK");

						for x in 0..GRID_SIZE {
							for y in 0..GRID_SIZE {
								if state.cells.get(&(x, y)).is_none() && (neighbours(&state.cells, (x, y)).len() == 3) {
									living.push((x, y));
								}
								if state.cells.get(&(x, y)).is_some() {
									let num_neighbours = neighbours(&state.cells, (x, y)).len();
									match num_neighbours {
										0..=1 => dead.push((x, y)),
										2..=3 => {},
										_ => dead.push((x, y)),
									}
								}
							}
						}

						for point in living.into_iter() {
							let _ = state.cells.insert(point);
						}
						for point in dead.into_iter() {
							let _ = state.cells.remove(&point);
						}
					}
				})
			}
		});
	}

	dom::mount(
		Rc::clone(&state_rc),
		Cmp::new(move |rc| {
			primitives::div(
				children![
					start_button(&rc).as_node(),
					randomize_button(&rc).as_node(),
					container(&rc).as_node(),
				],
				attrs![],
				|_| {},
			)
		}),
	);
}
