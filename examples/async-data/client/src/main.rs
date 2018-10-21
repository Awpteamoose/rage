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
use strum_macros::AsStaticStr;
use shared::{TestArg, Method};
use serde::{Serialize, Deserialize};

thread_local! {
	pub static STATE: StateLock<MyState> = StateLock::default();
}

#[derive(Debug)]
pub struct MyState {
	rng: RefCell<SmallRng>,
}

impl Default for MyState {
	fn default() -> Self {
		Self {
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

fn fetch<V: Serialize + Deserialize<'static>>(method: Method, arg: &V) -> PromiseFuture<String> {
	#[allow(clippy::result_unwrap_used)]
	js!(return fetch("http://localhost:8000" + @{method.as_str()}, { method: "POST", body: Uint8Array.from(@{serde_cbor::to_vec(arg).unwrap()}) }).then((r)=>r.text());).try_into().unwrap()
}

async fn print(message: &str) {
	// Waits for 2000 milliseconds
	await!(wait(2000));
	console!(log, message);
}

#[allow(clippy::useless_let_if_seq)]
async fn future_main() -> Result<(), Error> {
	// Runs Futures synchronously
	// await!(print("Hello"));
	// await!(print("There"));

	// {
	//     let a = print("Test 1");
	//     let b = print("Test 2");

	//     // Runs multiple Futures in parallel
	//     join!(a, b);

	//     console!(log, "Done");
	// }

	{
		let a = fetch(Method::TestMethod, &TestArg { prop1: 15, prop2: "nigg".to_owned() });
		let b = fetch(Method::TestMethod, &TestArg { prop1: 5, prop2: "bigg".to_owned() });

		// Runs multiple Futures (which can error) in parallel
		let (a, b) = try_join!(a, b)?;

		console!(log, a, b);
	}

	Ok(())
}

fn root() -> Element {
	primitives::div(
		children![
			"I have a big nose",
			primitives::div(children!["wow"], attrs![], events![]),
			"I have a big nose",
			primitives::div(children!["that's a big nose"], attrs![], events![]),
		],
		attrs![],
		events![],
	)
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	spawn_local(unwrap_future(future_main()));

	vdom::mount(&STATE, root);
}
