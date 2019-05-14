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

use http::method::Method as HttpMethod;
use serde_derive::{Deserialize, Serialize};
use strum_macros::AsStaticStr;

#[derive(Debug)]
pub enum Method {
	TestMethod,
}

impl Method {
	pub fn as_str(&self) -> &'static str {
		match self {
			Method::TestMethod => "/api/test-method",
		}
	}

	pub fn method(&self) -> HttpMethod {
		match self {
			Method::TestMethod => HttpMethod::POST,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestArg {
	pub prop1: u32,
	pub prop2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReply {
	pub some: bool,
	pub other: String,
}
