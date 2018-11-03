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

#[macro_export]
macro_rules! children {
	() => {
		vec![]
	};
	($($e: expr),+$(,)*) => {
		std::vec![$($e.into(),)+]
	};
}

#[macro_export]
macro_rules! attrs {
	() => {
		maplit::hashmap![]
	};
	($($k: expr => $v: expr),+$(,)*) => {
		hashmap![$($k.into() => $v.into(),)+]
	};
}

#[macro_export]
macro_rules! events {
	() => {
		std::vec![]
	};
	($($e: expr),+$(,)*) => {
		vec![$(<Box<dyn Fn(&_)>>::into(Box::new($e)),)+]
	};
}

#[macro_export]
macro_rules! enclose {
	(($($x:ident),*) $y:expr) => {{
		$(let $x = $x.clone();)*
		$y
	}};
}

#[macro_use]
pub mod primitives;
pub mod cmp;
mod styled;
pub mod vdom;

pub use self::{
	cmp::{Component, Tracked},
	styled::styled,
};
pub use stdweb;
