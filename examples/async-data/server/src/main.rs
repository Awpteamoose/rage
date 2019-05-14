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

use actix_web::{
	Error as ActixError,
	HttpResponse,
};
use futures::prelude::*;
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use shared::{Method, TestArg, TestReply};

#[derive(Deserialize)]
struct Config {
	address: String,
	port: String,
	source: String,
}

lazy_static! {
	static ref CONFIG: Config =
		toml::from_str(&std::fs::read_to_string("Config.toml").expect("can't read config file")).expect("can't deserialize config file");
}

async fn test_method(body: bytes::Bytes) -> Result<HttpResponse, ActixError> {
	let arg: TestArg = serde_cbor::from_slice(&body).expect("client sent garbage");
	println!("arg: {:#?}", &arg);
	let reply = TestReply {
		some: true,
		other: "boop".to_owned(),
	};
	let reply_vec: bytes::Bytes = serde_cbor::to_vec(&reply).expect("can't serialize").into();
	Ok(reply_vec.into())
}

fn main() -> Result<(), std::io::Error> {
	use actix_web::middleware::Logger;

	std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	actix_web::HttpServer::new(||
		actix_web::App::new()
			.wrap(Logger::default())
			.wrap(Logger::new("%a"))
			// .route("/favicon.ico", HttpMethod::GET, |_: HttpRequest| NamedFile::open("public/favicon.ico"))
			// .handler("/public", StaticFiles::new("public/").expect("can't serve public/"))
			.route(Method::TestMethod.as_str(), actix_web::web::method(Method::TestMethod.method()).to_async(|x| test_method(x).boxed_local().compat()))
			.service(actix_files::Files::new("/", &CONFIG.source).index_file("index.html"))
	)
	.bind(format!("{}:{}", CONFIG.address, CONFIG.port))?
	.run()
}
