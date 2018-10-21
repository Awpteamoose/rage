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
#![recursion_limit = "128"]
#![allow(unreachable_pub)]
#![feature(try_from, try_trait, never_type)]
#![feature(async_await, await_macro, futures_api, pin)]

#[macro_use] extern crate rage;

use maplit::*;
use std::collections::HashSet;
use std::ops::Add;
use std::cell::RefCell;
use rand::prelude::*;
use rage::{
	stdweb::{
		self,
		__internal_console_unsafe,
		__js_raw_asm,
		_js_impl,
		console,
		js,
		spawn_local,
		unstable::TryInto,
		unwrap_future,
		web::{
			error::Error,
			event,
			wait,
		},
		PromiseFuture,
		traits::*,
		unstable::TryFrom,
	},
	futures::{join, try_join},
	cmp::*,
	styled::styled,
	vdom::Element,
	primitives,
	vdom,
};

thread_local! {
	pub static STATE: StateLock<MyState> = StateLock::default();
}

const CELL_SIZE: u32 = 12;
type Cell = ToroidalPoint;
type Cells = HashSet<ToroidalPoint>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ToroidalPoint(u32, u32);

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add<(i32, i32)> for ToroidalPoint {
	type Output = Self;

	fn add(self, other: (i32, i32)) -> Self {
		STATE.view(|state| {
			let grid_size = state.grid_size;

			let x = if other.0 > 0 {
				let x = self.0 + other.0 as u32;
				if x >= grid_size { x - grid_size }
				else { x }
			} else {
				let other_x = other.0.abs() as u32;
				let x = self.0.checked_sub(other_x);
				if let Some(x) = x { x } else { grid_size - other_x + self.0 }
			};
			let y = if other.1 > 0 {
				let y = self.1 + other.1 as u32;
				if y >= grid_size { y - grid_size }
				else { y }
			} else {
				let other_y = other.1.abs() as u32;
				let y = self.1.checked_sub(other_y);
				if let Some(y) = y { y } else { grid_size - other_y + self.1 }
			};

			ToroidalPoint(x, y)
		})
	}
}

fn neighbours(cells: &Cells, point: Cell) -> Vec<Cell> {
	vec![
		point + (-1, -1),
		point + (-1, 0),
		point + (-1, 1),
		point + (0, -1),
		point + (0, 1),
		point + (1, -1),
		point + (1, 0),
		point + (1, 1),
	]
		.into_iter()
		.filter(|x| cells.contains(&x))
		.collect::<Vec<_>>()
}

#[derive(Debug)]
pub struct MyState {
	cells: Cells,
	running: bool,
	grid_size: u32,
	rng: RefCell<SmallRng>,
}

impl Default for MyState {
	fn default() -> Self {
		Self {
			cells: Cells::default(),
			running: false,
			grid_size: 75,
			rng: {
				let mut bytes: [u8; 16] = [0; 16];
				let seed: [u8; 8] = unsafe { std::mem::transmute(stdweb::web::Date::new().get_time()) };
				for (index, byte) in seed.iter().enumerate() {
					bytes[index] = *byte;
					bytes[index + 8] = *byte;
				}
				RefCell::new(rand::rngs::SmallRng::from_seed(bytes))
			},
		}
	}
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

fn cells() -> Vec<Element> {
	STATE.lock(|lock| {
		let mut divs = Vec::new();
		let grid_size = lock.view().grid_size;

		for x in 0..grid_size {
			for y in 0..grid_size {
				divs.push(primitives::div(
					children![],
					attrs![
						"class" => styled(&lock, &format!(r#"
							border: 1px solid black;
							background-color: {color};
							box-sizing: content-box;
						"#,
							color = if lock.view().cells.get(&ToroidalPoint(x, y)).is_some() { "black" } else { "white" }
						)),
					],
					events![
						move |_: event::ClickEvent| {
							STATE.update(|state| {
								console!(log, "click start");

								if state.cells.get(&ToroidalPoint(x, y)).is_some() { let _ = state.cells.remove(&ToroidalPoint(x, y)); }
								else { let _ = state.cells.insert(ToroidalPoint(x, y)); };
								console!(log, "click end");
							});
						},
					],
				));
			}
		}

		divs
	})
}

fn start_button() -> Element {
	STATE.view(|state| {
		primitives::input(
			children![],
			attrs![
				"type" => "button",
				"value" => if state.running { "stop" } else { "start" },
			],
			events![
				move |_: event::ClickEvent| {
					STATE.update(|state| {
						state.running = !state.running;
					});
				},
			],
		)
	})
}

fn randomize_button() -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => "randomize",
		],
		events![
			move |_: event::ClickEvent| {
				STATE.update(|state| {
					let grid_size = state.grid_size;

					for x in 0..grid_size {
						for y in 0..grid_size {
							if state.rng.borrow_mut().gen_bool(0.5) {
								let _ = state.cells.insert(ToroidalPoint(x, y));
							} else {
								let _ = state.cells.remove(&ToroidalPoint(x, y));
							}
						}
					}
				})
			},
		],
	)
}

fn clear_button() -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => "clear",
		],
		events![
			move |_: event::ClickEvent| {
				STATE.update(|state| {
					state.cells.clear();
				});
			},
		],
	)
}

fn size() -> Element {
	STATE.view(|state| {
		primitives::div(
			children![
				"Grid size",
				primitives::input(
					children![],
					attrs![
						"type" => "range",
						"min" => "10",
						"max" => "100",
						"style" => "width: 1000px",
						"value" => state.grid_size.to_string(),
					],
					events![
						move |e: event::InputEvent| {
							STATE.update(|state| {
								let value = stdweb::web::html_element::InputElement::try_from(e.current_target().unwrap()).unwrap().raw_value();
								state.grid_size = value.parse().unwrap();
							});
						},
					],
				)
			],
			attrs![],
			events![],
		)
	})
}

fn container() -> Element {
	STATE.lock(|lock| {
		primitives::div(
			cells(),
			attrs![
				"class" => styled(&lock, &format!(r#"
					user-select: none;
					display: grid;
					grid-template-columns: repeat({grid_size}, {cell_size}px);
					grid-template-rows: repeat({grid_size}, {cell_size}px);
				"#, grid_size = lock.view().grid_size, cell_size = CELL_SIZE)),
			],
			events![],
		)
	})
}

fn tick() {
	STATE.lock(|lock| {
		let mut living = Vec::new();
		let mut dead = Vec::new();
		{
			let state = &lock.view();
			let grid_size = state.grid_size;

			for x in 0..grid_size {
				for y in 0..grid_size {
					if state.cells.get(&ToroidalPoint(x, y)).is_none() && (neighbours(&state.cells, ToroidalPoint(x, y)).len() == 3) {
						living.push(ToroidalPoint(x, y));
					}
					if state.cells.get(&ToroidalPoint(x, y)).is_some() {
						let num_neighbours = neighbours(&state.cells, ToroidalPoint(x, y)).len();
						match num_neighbours {
							0..=1 => dead.push(ToroidalPoint(x, y)),
							2..=3 => {},
							_ => dead.push(ToroidalPoint(x, y)),
						}
					}
				}
			}
		}

		let state = &mut lock.update();
		for point in living.into_iter() {
			let _ = state.cells.insert(point);
		}
		for point in dead.into_iter() {
			let _ = state.cells.remove(&point);
		}
	});
}

fn root() -> Element {
	STATE.view(|state| {
		if state.running {
			let _ = stdweb::web::window().request_animation_frame(|_| tick());
		}
	});

	primitives::div(
		children![
			"I have a big nose",
			primitives::div(children!["wow"], attrs![], events![]),
			"I have a big nose",
			primitives::div(children!["that's a big nose"], attrs![], events![]),
			start_button(),
			randomize_button(),
			clear_button(),
			size(),
			container(),
		],
		attrs![],
		events![],
	)
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	// spawn_local(unwrap_future(future_main()));

	vdom::mount(&STATE, root);
}
