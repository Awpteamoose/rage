#![deny(rust_2018_idioms)]
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
	clippy::shadow_unrelated,

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
	unused_results
)]
#![cfg_attr(not(debug_assertions), warn(
	// Restriction
	clippy::use_debug,
	clippy::print_stdout,
	clippy::unimplemented,
))]
#![allow(clippy::result_unwrap_used, clippy::option_unwrap_used)]
#![feature(try_from, try_trait, never_type, tool_lints, set_stdio)]
#![recursion_limit = "128"]

mod primitives;
mod styled;

use self::{primitives::*, styled::*};
use std::{
	cell::RefCell,
	collections::{hash_map::DefaultHasher, HashMap},
	hash::Hasher,
	rc::Rc,
};
use stdweb::{
	js, _js_impl, __js_raw_asm,
	console, __internal_console_unsafe,
	traits::*,
	unstable::TryFrom,
	web::{document, event, HtmlElement, Node},
};
// use maplit::*;

#[derive(Default, Debug)]
pub struct State {
	some_value: i32,
}

pub struct StateLock {
	root: Node,
	style: HtmlElement,
	mount: Rc<RefCell<Vec<Box<dyn Component>>>>,
	styles: Rc<RefCell<HashMap<String, String>>>,
	state: State,
}

impl Default for StateLock {
	fn default() -> Self {
		Self {
			root: Node::from(document().create_element("div").unwrap()),
			style: HtmlElement::try_from(document().create_element("style").unwrap()).unwrap(),
			mount: Rc::new(RefCell::new(Vec::new())),
			styles: Rc::new(RefCell::new(HashMap::new())),
			state: State::default(),
		}
	}
}

impl StateLock {
	fn update(arc: &mut StateRc, f: impl Fn(&mut State)) {
		f(&mut arc.borrow_mut().state);
		update_dom(arc);
	}
}

pub type StateRc = Rc<RefCell<StateLock>>;

pub trait Component {
	fn render(&mut self, _: StateRc) -> Node;
	fn children(&mut self) -> &mut Vec<Box<dyn Component>> { unimplemented!() }
	fn attributes(&mut self) -> &mut HashMap<String, String> { unimplemented!() }
}

fn update_node(parent: &mut Node, old: &mut Option<Node>, new: &Option<Node>) {
	match (old, new) {
		(None, Some(node)) => {
			parent.append_child(node);
		},
		(Some(node), None) => {
			let _ = parent.remove_child(node).unwrap();
		},
		(Some(old_node), Some(new_node)) if !Node::is_equal_node(old_node, new_node) => {
			if !old_node.has_child_nodes() || !new_node.has_child_nodes() {
				let _ = parent.replace_child(new_node, old_node).unwrap();
			} else {
				let old_node_children = old_node.child_nodes();
				let new_node_children = new_node.child_nodes();
				let min = u32::min(old_node_children.len(), new_node_children.len());

				for i in 0 .. min {
					update_node(old_node, &mut old_node_children.item(i), &new_node_children.item(i));
				}

				// less nodes in new than old -> remove nodes
				for i in min .. old_node_children.len() {
					let _ = parent.remove_child(&old_node_children.item(i).unwrap()).unwrap();
				}

				// less nodes in old than new -> add nodes
				for i in min .. new_node_children.len() {
					parent.append_child(&new_node_children.item(i).unwrap());
				}
			}
		},
		_ => (),
	}
}

fn update_dom(state: &StateRc) {
	let mut nodes = Vec::new();
	for cmp in state.borrow().mount.borrow_mut().iter_mut() {
		nodes.push(cmp.render(Rc::clone(&state)));
	}

	let StateLock { style, root, styles, .. } = &mut state.borrow_mut() as &mut StateLock;

	styles.borrow_mut().clear();

	style.set_text_content(&styles.borrow_mut().iter().fold(String::new(), |acc, (class, style)| {
		acc + &format!(".{} {{ {} }}", class, style)
	}));

	let root_children = root.child_nodes();

	let mut with_index: Vec<_> = nodes.into_iter().enumerate().collect();

	while let Some((index, node)) = with_index.pop() {
		update_node(root, &mut root_children.item(index as u32), &Some(node));
	}
}

fn main() {
	let state: StateRc = StateRc::default();
	{
		let state_write = &mut state.borrow_mut() as &mut StateLock;

		// let test_div2 = Styled {
		//     inner: Div::new(HashMap::new(), vec![Box::new("pidoir")], Rc::default()),
		//     get_css: |_, state: &State| format!("width: {}px", state.some_value),
		// };

		// poop! [
		//     Div[
		//         []
		//         [
		//             Div[
		//                 []
		//                 ["div1"]
		//                 []
		//             ]
		//             Div[
		//                 []
		//                 ["div2"]
		//                 []
		//             ]
		//             Div[
		//                 []
		//                 [|state: StateRc| format!("more {}", state.borrow().state.some_value)]
		//                 [
		//                     click => move |_| {
		//                         StateLock::update(&mut new_state, move |s| {
		//                             s.some_value += 1;
		//                         });
		//                     },
		//                 ]
		//             ]
		//         ]
		//         []
		//     ]
		// ];

		let mut new_state = Rc::clone(&state);
		let test_div = Div::new(
			HashMap::new(),
			vec![
				Box::new(Div::new(HashMap::new(), vec![
					Box::new("div1"),
				], Rc::default())),
				Box::new(Div::new(HashMap::new(), vec![
					Box::new("div2"),
				], Rc::default())),
				Box::new(Div::new(HashMap::new(), vec![
					Box::new(|state: StateRc| format!("more {}", state.borrow().state.some_value)),
				], Rc::new(RefCell::new(Some(Box::new(move |_| {
					StateLock::update(&mut new_state, move |s| {
						s.some_value += 1;
					});
				})))))),
			],
			Rc::default(),
		);

		state_write.mount.borrow_mut().push(Box::new(test_div));

		document().head().unwrap().append_child(&state_write.style);
		document().body().unwrap().append_child(&state_write.root);
	}

	update_dom(&state);
}
