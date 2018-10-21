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

#[macro_use]
extern crate rage;

use maplit::*;
use rage::{
	cmp::*,
	primitives,
	stdweb::{
		__js_raw_asm,
		_js_impl,
		js,
		spawn_local,
		unstable::TryInto,
		unwrap_future,
		web::{error::Error, wait, TypedArray},
		PromiseFuture,
	},
	vdom::{self, Element},
};
use serde::{de::DeserializeOwned, Serialize};
use shared::{Method, TestArg, TestReply};

thread_local! {
	pub static STATE: StateLock<MyState> = StateLock::default();
}

#[derive(Default, Debug)]
pub struct MyState {
	pub reply: Option<TestReply>,
}

fn fetch<V: Serialize + DeserializeOwned>(method: &Method, arg: &V) -> PromiseFuture<TypedArray<u8>> {
	let http_method = method.method();
	#[allow(clippy::result_unwrap_used)]
	js!(
		return fetch(
			@{method.as_str()},
			{
				method: @{http_method.as_str()},
				body: Uint8Array.from(@{serde_cbor::to_vec(arg).unwrap()}),
			},
		)
			.then((r) => r.arrayBuffer())
			.then((b) => new Uint8Array(b))
	)
	.try_into()
	.unwrap()
}

async fn method<Arg: Serialize + DeserializeOwned, Reply: DeserializeOwned>(method: Method, arg: Arg) -> Result<Reply, Error> {
	let res = await!(fetch(&method, &arg))?;
	let vec: Vec<u8> = res.into();
	let rep: Reply = serde_cbor::from_slice(&vec).expect("server replied with garbage");
	Ok(rep)
}

#[allow(clippy::useless_let_if_seq)]
async fn future_main() -> Result<(), Error> {
	await!(wait(2000));
	let reply: TestReply = await!(method(
		Method::TestMethod,
		TestArg {
			prop1: 15,
			prop2: "gigg".to_owned()
		}
	))?;
	STATE.update(|state| {
		state.reply = Some(reply);
	});

	Ok(())
}

fn root() -> Element {
	STATE.view(|state| primitives::div(children![format!("TestReply: {:?}", state.reply),], attrs![], events![]))
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	spawn_local(unwrap_future(future_main()));

	vdom::mount(&STATE, root);
}
