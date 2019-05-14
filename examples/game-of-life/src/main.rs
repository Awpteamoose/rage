#![deny(rust_2018_idioms, unused_must_use)]
#![warn(
	clippy::pedantic,
	clippy::style,
	clippy::complexity,
	clippy::perf,
	clippy::correctness,
	clippy::clone_on_ref_ptr,
	clippy::float_cmp_const,
	clippy::option_unwrap_used,
	clippy::result_unwrap_used,
	clippy::wrong_pub_self_convention,
	clippy::shadow_reuse,
	clippy::missing_const_for_fn,
	anonymous_parameters,
	bare_trait_objects,
	missing_copy_implementations,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates,
	unused_import_braces,
	unused_qualifications,
	unused_results
)]
#![cfg_attr(
	not(debug_assertions),
	warn(clippy::use_debug, clippy::print_stdout, clippy::unimplemented)
)]

#[macro_export]
macro_rules! println {
	($($arg: tt),+$(,)*) => {
		use rage::stdweb::{
			console,
			js,
		};

		console!(log, format!($($arg,)+));
	};
}

#[macro_export]
macro_rules! eprintln {
	($($arg: tt),+$(,)*) => {
		use rage::stdweb::{
			console,
			js,
		};

		console!(error, format!($($arg,)+));
	};
}

#[macro_use]
extern crate rage;

use maplit::*;
use rage::{
	cmp::*,
	primitives::html as primitives,
	stdweb::{self, __internal_console_unsafe, _js_impl, console, js, traits::*, unstable::TryFrom, web::event},
	styled,
	vdom::{self, Element},
	Tracked,
};
use rand::prelude::*;
use std::{cell::RefCell, collections::HashSet, ops::Add};

mod styles {
	use rage::styled;

	lazy_static::lazy_static! {
		pub static ref ALIVE: String = styled("
			.& {
				border: 1px solid black;
				background-color: black;
				box-sizing: content-box;
			}
		");
		pub static ref DEAD: String = styled("
			.& {
				border: 1px solid black;
				background-color: white;
				box-sizing: content-box;
			}
		");
	}
}

thread_local! {
	pub static STATE: Tracked<State> = Tracked::new(State::default());
}

const CELL_SIZE: u32 = 12;
type Cell = ToroidalPoint;
type Cells = HashSet<ToroidalPoint>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ToroidalPoint(u32, u32);

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add<(i32, i32, u32)> for ToroidalPoint {
	type Output = Self;

	fn add(self, other: (i32, i32, u32)) -> Self {
		let grid_size = other.2;

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
	}
}

fn neighbours(cells: &Cells, point: Cell, s: u32) -> Vec<Cell> {
	vec![
		point + (-1, -1, s),
		point + (-1, 0, s),
		point + (-1, 1, s),
		point + (0, -1, s),
		point + (0, 1, s),
		point + (1, -1, s),
		point + (1, 0, s),
		point + (1, 1, s),
	]
		.into_iter()
		.filter(|x| cells.contains(&x))
		.collect::<Vec<_>>()
}

#[derive(Debug)]
pub struct State {
	running: bool,
	grid_size: u32,
	cells: Cells,
}

impl Default for State {
	fn default() -> Self {
		Self {
			running: false,
			grid_size: 75,
			cells: Cells::default(),
		}
	}
}

fn cells(state: &Tracked<State>) -> Vec<Element> {
	let mut divs = Vec::new();
	let grid_size = state.view().grid_size;

	for x in 0..grid_size {
		for y in 0..grid_size {
			divs.push(primitives::div(
				children![],
				attrs![
					"class" => if state.view().cells.get(&ToroidalPoint(x, y)).is_some() { &styles::ALIVE as &str } else { &styles::DEAD as &str },
				],
				events![
					enclose!{(state) move |_: &event::ClickEvent| {
						let mut state = state.update();
						console!(log, "click start");

						if state.cells.get(&ToroidalPoint(x, y)).is_some() { let _ = state.cells.remove(&ToroidalPoint(x, y)); }
						else { let _ = state.cells.insert(ToroidalPoint(x, y)); };
						console!(log, "click end");
					}},
				],
			));
		}
	}

	divs
}

fn start_button(state: &Tracked<State>) -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => if state.view().running { "stop" } else { "start" },
		],
		events![
			enclose!{(state) move |_: &event::ClickEvent| {
				let mut state = state.update();
				state.running = !state.running;
			}},
		],
	)
}

fn randomize_button(state: &Tracked<State>) -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => "randomize",
		],
		events![
			enclose!{(state) move |_: &event::ClickEvent| {
				let mut state = state.update();
				let grid_size = state.grid_size;

				for x in 0..grid_size {
					for y in 0..grid_size {
						if thread_rng().gen_bool(0.5) {
							let _ = state.cells.insert(ToroidalPoint(x, y));
						} else {
							let _ = state.cells.remove(&ToroidalPoint(x, y));
						}
					}
				}
			}},
		],
	)
}

fn clear_button(state: &Tracked<State>) -> Element {
	primitives::input(
		children![],
		attrs![
			"type" => "button",
			"value" => "clear",
		],
		events![
			enclose!{(state) move |_: &event::ClickEvent| state.update().cells.clear()},
		],
	)
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn size(state: &Tracked<State>) -> Element {
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
					"value" => state.view().grid_size.to_string(),
				],
				events![
					enclose!{(state) move |e: &event::InputEvent| {
						let mut state = state.update();
						let value = stdweb::web::html_element::InputElement::try_from(e.target().unwrap()).unwrap().raw_value();
						state.grid_size = value.parse().unwrap();
					}},
				],
			)
		],
		attrs![],
		events![],
	)
}

fn container(state: &Tracked<State>) -> Element {
	primitives::div(
		cells(state),
		attrs![
			"class" => styled(r#"
				.& {
					user-select: none;
					display: grid;
				}
			"#),
			"style" => format!(r#"
				grid-template-columns: repeat({grid_size}, {cell_size}px);
				grid-template-rows: repeat({grid_size}, {cell_size}px);
			"#, grid_size = state.view().grid_size, cell_size = CELL_SIZE),
		],
		events![],
	)
}

fn tick(state: &Tracked<State>) {
	if !state.view().running { return; }
	let grid_size = state.view().grid_size;

	let mut state = state.update();
	let mut living = Vec::new();
	let mut dead = Vec::new();

	for x in 0..grid_size {
		for y in 0..grid_size {
			if state.cells.get(&ToroidalPoint(x, y)).is_none() && (neighbours(&state.cells, ToroidalPoint(x, y), state.grid_size).len() == 3) {
				living.push(ToroidalPoint(x, y));
			}
			if state.cells.get(&ToroidalPoint(x, y)).is_some() {
				let num_neighbours = neighbours(&state.cells, ToroidalPoint(x, y), state.grid_size).len();
				match num_neighbours {
					0..=1 => dead.push(ToroidalPoint(x, y)),
					2..=3 => {},
					_ => dead.push(ToroidalPoint(x, y)),
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

fn root() -> Element {
	STATE.with(|state| {
		tick(state);

		primitives::div(
			children![
				"I have a big nose",
				primitives::div(children!["wow"], attrs![], events![]),
				"I have a big nose",
				primitives::div(children!["that's a big nose"], attrs![], events![]),
				start_button(state),
				randomize_button(state),
				clear_button(state),
				size(state),
				container(state),
			],
			attrs![],
			events![],
		)
	})
}

fn main() {
	std::panic::set_hook(Box::new(|i| {
		// TODO: in release
		// js!(document.location.reload());
		eprintln!("{}", i);
	}));
	rage::stdweb::web::document().set_title("Game of Life");

	STATE.with(|state| {
		let mut state = state.update();
		state.running = true;
		let grid_size = state.grid_size;

		for x in 0..grid_size {
			for y in 0..grid_size {
				if thread_rng().gen_bool(0.5) {
					let _ = state.cells.insert(ToroidalPoint(x, y));
				} else {
					let _ = state.cells.remove(&ToroidalPoint(x, y));
				}
			}
		}
	});

	vdom::mount(root);
}
