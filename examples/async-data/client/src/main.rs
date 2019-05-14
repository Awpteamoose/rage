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
#![feature(async_await, await_macro)]

#[macro_use]
extern crate rage;

use maplit::*;
use rage::{
	cmp::*,
	primitives,
	stdweb::{
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
	pub static STATE: Tracked<MyState> = Tracked::new(MyState::default());
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

	STATE.with(|state| state.update().reply = Some(reply));

	Ok(())
}

fn root() -> Element {
	STATE.with(|state| primitives::html::div(children![format!("TestReply: {:?}", state.view().reply),], attrs![], events![]))
}

#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
fn main() {
	spawn_local(unwrap_future(future_main()));

	vdom::mount(root);
}
