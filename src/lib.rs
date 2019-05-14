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
macro_rules! children {
	() => {
		vec![]
	};
	($($e: expr),+$(,)*) => {
		vec![$($e.into(),)+]
	};
}

#[macro_export]
macro_rules! attrs {
	() => {{
		use $crate::maplit::hashmap;
		hashmap![]
	}};
	($($k: expr => $v: expr),+$(,)*) => {{
		use $crate::maplit::hashmap;
		hashmap![$($k.into() => $v.into(),)+]
	}};
}

#[macro_export]
macro_rules! events {
	() => {
		vec![]
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
	styled::{styled, append_css},
};
pub use stdweb;
pub use maplit;
