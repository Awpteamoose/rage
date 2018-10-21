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

use actix_web::{
	fs::{NamedFile, StaticFiles},
	App,
	HttpMessage,
	HttpRequest,
	HttpResponse,
	Error as ActixError,
};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use futures::prelude::*;
use futures::compat::*;
use futures_01::Future as Future01;
use shared::{TestArg, Method, TestReply};

#[derive(Deserialize)]
struct Config {
	address: String,
	port: String,
}

lazy_static! {
	static ref CONFIG: Config = toml::from_str(&std::fs::read_to_string("Config.toml").expect("can't read config file")).expect("can't deserialize config file");
}

async fn test_method(req: HttpRequest) -> Result<HttpResponse, ActixError> {
	println!("req: {:#?}", req);
	let body: bytes::Bytes = await!(Compat01As03::new(req.body())).unwrap();
	let arg: TestArg = serde_cbor::from_slice(&body).expect("client sent garbage");
	println!("arg: {:#?}", &arg);
	let reply = TestReply { some: true, other: "boop".to_owned() };
	let reply_vec: bytes::Bytes = serde_cbor::to_vec(&reply).expect("can't serialize").into();
	Ok(reply_vec.into())
}

fn main() {
	use actix_web::middleware::Logger;

	std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	let sys = actix::System::new("rage-server");

	let _ = actix_web::server::new(|| {
		App::new()
			.middleware(Logger::default())
			.middleware(Logger::new("%a"))
			// .route("/favicon.ico", HttpMethod::GET, |_: HttpRequest| NamedFile::open("public/favicon.ico"))
			// .handler("/public", StaticFiles::new("public/").expect("can't serve public/"))
			.route(
				Method::TestMethod.as_str(),
				Method::TestMethod.method(),
				|req: HttpRequest| -> Box<dyn Future01<Item = _, Error = _>> { Box::new(test_method(req).boxed().compat()) },
			)
			.handler("/",
				StaticFiles::new("target/deploy").expect("can't serve")
				.default_handler(|_: &HttpRequest| NamedFile::open("target/deploy/index.html"))
			)
		})
		.bind(format!("{}:{}", CONFIG.address, CONFIG.port))
		.expect("can't bind to addres")
		.start();

	let _ = sys.run();
}
