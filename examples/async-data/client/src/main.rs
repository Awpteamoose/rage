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
			TypedArray,
			ArrayBuffer,
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
use shared::{TestArg, Method, TestReply};
use serde::{Serialize, Deserialize};

thread_local! {
	pub static STATE: StateLock<MyState> = StateLock::default();
}

#[derive(Default, Debug)]
pub struct MyState {
	pub some_value: u32,
}

fn fetch<V: Serialize + Deserialize<'static>>(method: &Method, arg: &V) -> PromiseFuture<TypedArray<u8>> {
	#[allow(clippy::result_unwrap_used)]
	js!(
		return fetch(
			"http://localhost:8000" + @{method.as_str()},
			{
				method: "POST",
				body: Uint8Array.from(@{serde_cbor::to_vec(arg).unwrap()}),
			},
		)
			.then((r) => r.arrayBuffer())
			.then((b) => new Uint8Array(b))
	).try_into().unwrap()
}

#[allow(clippy::useless_let_if_seq)]
async fn future_main() -> Result<(), Error> {
	{
		let a = fetch(&Method::TestMethod, &TestArg { prop1: 15, prop2: "gigg".to_owned() });
		let b = fetch(&Method::TestMethod, &TestArg { prop1: 5, prop2: "bigg".to_owned() });

		// Runs multiple Futures (which can error) in parallel
		let (a, b) = try_join!(a, b)?;

		let v1: Vec<u8> = a.into();
		let reply1: TestReply = serde_cbor::from_slice(&v1).unwrap();

		console!(log, format!("{:?}, {:?}", &reply1, b));
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
