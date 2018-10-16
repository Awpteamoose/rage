use std::collections::HashMap;
use stdweb::{
	traits::*,
	web::{document, Element, INode, Node, event},
};
use strum_macros::AsStaticStr;

// TODO: I could skip the Event bit, but concat_idents! doesn't work properly ¯\_(ツ)_/¯
macro_rules! __event_idents {
	($m: ident, $arg1: ident, $arg2: ident) => {$m![$arg1, $arg2,
		AuxClickEvent, BlurEvent, ChangeEvent,
		ClickEvent, ContextMenuEvent, DoubleClickEvent,
		DragDropEvent, DragEndEvent, DragEnterEvent,
		DragEvent, DragExitEvent, DragLeaveEvent,
		DragOverEvent, DragStartEvent, FocusEvent,
		GamepadConnectedEvent, GamepadDisconnectedEvent, GotPointerCaptureEvent,
		HashChangeEvent, InputEvent, KeyDownEvent,
		KeyPressEvent, KeyUpEvent, LoadEndEvent,
		LoadStartEvent, LostPointerCaptureEvent, MouseDownEvent,
		MouseEnterEvent, MouseLeaveEvent, MouseMoveEvent,
		MouseOutEvent, MouseOverEvent, MouseUpEvent,
		MouseWheelEvent, PointerCancelEvent, PointerDownEvent,
		PointerEnterEvent, PointerLeaveEvent, PointerLockChangeEvent,
		PointerLockErrorEvent, PointerMoveEvent, PointerOutEvent,
		PointerOverEvent, PointerUpEvent, PopStateEvent,
		ProgressAbortEvent, ProgressErrorEvent, ProgressEvent,
		ProgressLoadEvent, ReadyStateChangeEvent, ResizeEvent,
		ResourceAbortEvent, ResourceErrorEvent, ResourceLoadEvent,
		ScrollEvent, SelectionChangeEvent, SocketCloseEvent,
		SocketErrorEvent, SocketMessageEvent, SocketOpenEvent,
		SubmitEvent,
	];};
}

macro_rules! __events {
	(skip, skip, $($name: ident),+$(,)*) => {
		#[allow(missing_debug_implementations, clippy::pub_enum_variant_names)]
		pub enum EventHandler {
			$($name(Box<dyn Fn(event::$name)>),)+
		}

		$(
			impl Into<EventHandler> for Box<dyn Fn(event::$name)> {
				fn into(self) -> EventHandler {
					EventHandler::$name(self)
				}
			}
		)+
	}
}

__event_idents![__events, skip, skip];

macro_rules! __event_listeners {
	($handler: expr, $element: expr, $($name: ident),+$(,)*) => {
		match $handler {
			$(
				EventHandler::$name(f) => {
					let _ = $element.add_event_listener(move |e: event::$name| f(e));
				},
			)+
		}
	};
}

macro_rules! __primitives {
	($($name: ident),+$(,)*) => {
		#[allow(non_camel_case_types, dead_code)]
		#[derive(Debug, AsStaticStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
		pub enum Tag {
			$($name,)+
		}

		$(
			#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, dead_code, non_snake_case, clippy::redundant_closure)]
			pub fn $name(
				children: &[&Node],
				attributes: &HashMap<&str, String>,
				event_handlers: Vec<EventHandler>,
			) -> Element {
				let element = document().create_element(stringify!($name)).unwrap();

				for child in children {
					element.append_child(*child);
				}

				for (name, value) in attributes.iter() {
					element.set_attribute(name, value).unwrap();
				}

				for handler in event_handlers.into_iter() {
					__event_idents![__event_listeners, handler, element];
				}

				element
			}
		)+
	};
}

__primitives!(
	a, abbr, address, area, article, aside, audio, b,
	base, bdi, bdo, big, blockquote, body, br, button,
	canvas, caption, circle, cite, clipPath, code, col, colgroup,
	data, datalist, dd, defs, del, details, dfn, dialog,
	div, dl, dt, ellipse, em, embed, fieldset, figcaption,
	figure, footer, foreignObject, form, g, h1, h2, h3,
	h4, h5, h6, head, header, hgroup, hr, html,
	i, iframe, image, img, input, ins, kbd, keygen,
	label, legend, li, line, linearGradient, link, main, map,
	mark, marquee, mask, menu, menuitem, meta, meter, nav,
	noscript, object, ol, optgroup, option, output, p, param,
	path, pattern, picture, polygon, polyline, pre, progress, prototype,
	q, radialGradient, rect, rp, rt, ruby, s, samp,
	script, section, select, small, source, span, stop, strong,
	style, sub, summary, sup, svg, table, tbody, td,
	text, textarea, tfoot, th, thead, time, title, tr,
	track, tspan, u, ul, var, video, wbr,
);

pub fn text_node(s: &str) -> Node { Node::from(document().create_text_node(s)) }

macro_rules! children {
	() => {
		&[]
	};
	($($e: expr),+$(,)*) => {
		&[$($e,)+]
	};
}

macro_rules! attrs {
	() => {
		&hashmap![]
	};
	($($k: expr => $v: expr),+$(,)*) => {
		&hashmap![$($k => $v.into(),)+]
	};
}

macro_rules! events {
	() => {
		vec![]
	};
	($($e: expr),+$(,)*) => {
		vec![$(<Box<dyn Fn(_)>>::into(Box::new($e)),)+]
	};
}
