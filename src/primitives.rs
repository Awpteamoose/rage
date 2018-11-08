use crate::vdom::Element;
use std::collections::HashMap;
use stdweb::web::event;
use strum_macros::AsStaticStr;

// TODO: I could skip the Event bit, but concat_idents! doesn't work properly ¯\_(ツ)_/¯
macro_rules! __event_idents {
	($m: ident, $arg1: ident, $arg2: ident) => {$m![$arg1, $arg2,
		AuxClickEvent,          BlurEvent,                 ChangeEvent,
		ClickEvent,             ContextMenuEvent,          DoubleClickEvent,
		DragDropEvent,          DragEndEvent,              DragEnterEvent,
		DragEvent,              DragExitEvent,             DragLeaveEvent,
		DragOverEvent,          DragStartEvent,            FocusEvent,
		GamepadConnectedEvent,  GamepadDisconnectedEvent,  GotPointerCaptureEvent,
		HashChangeEvent,        InputEvent,                KeyDownEvent,
		KeyPressEvent,          KeyUpEvent,                LoadEndEvent,
		LoadStartEvent,         LostPointerCaptureEvent,   MouseDownEvent,
		MouseEnterEvent,        MouseLeaveEvent,           MouseMoveEvent,
		MouseOutEvent,          MouseOverEvent,            MouseUpEvent,
		MouseWheelEvent,        PointerCancelEvent,        PointerDownEvent,
		PointerEnterEvent,      PointerLeaveEvent,         PointerLockChangeEvent,
		PointerLockErrorEvent,  PointerMoveEvent,          PointerOutEvent,
		PointerOverEvent,       PointerUpEvent,            PopStateEvent,
		ProgressAbortEvent,     ProgressErrorEvent,        ProgressEvent,
		ProgressLoadEvent,      ReadyStateChangeEvent,     /*ResizeEvent,*/
		ResourceAbortEvent,     ResourceErrorEvent,        ResourceLoadEvent,
		ScrollEvent,            SelectionChangeEvent,      SocketCloseEvent,
		SocketErrorEvent,       SocketMessageEvent,        SocketOpenEvent,
		SubmitEvent,
	];};
}

macro_rules! __events {
	(skip, skip, $($name: ident),+$(,)*) => {
		#[allow(missing_debug_implementations, clippy::pub_enum_variant_names)]
		pub enum EventHandler {
			$($name(Box<dyn Fn(&event::$name)>),)+
		}

		$(
			impl Into<EventHandler> for Box<dyn Fn(&event::$name)> {
				fn into(self) -> EventHandler {
					$crate::primitives::EventHandler::$name(self)
				}
			}
		)+
	}
}

#[derive(Debug, AsStaticStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum XmlElement {
	Raw(String),
	TextNode(String),
	Html(html::Tag),
	Svg(svg::Tag),
}

pub fn raw(s: String, attributes: HashMap<String, String>, event_handlers: Vec<EventHandler>) -> Element {
	Element::new(XmlElement::Raw(s), children![], attributes, event_handlers)
}

__event_idents![__events, skip, skip];

pub mod html {
	use super::{XmlElement, EventHandler};
	use crate::vdom::Element;
	use strum_macros::AsStaticStr;
	use std::collections::HashMap;

	macro_rules! __decl {
		($($name: ident),+$(,)*) => {
			#[allow(non_camel_case_types, dead_code)]
			#[derive(Debug, AsStaticStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
			pub enum Tag {
				$($name,)+
			}

			$(
				#[allow(dead_code, non_snake_case)]
				pub fn $name(
					children: Vec<Element>,
					attributes: HashMap<String, String>,
					event_handlers: Vec<EventHandler>,
				) -> Element {
					Element::new(XmlElement::Html(Tag::$name), children, attributes, event_handlers)
				}
			)+
		};
	}

	__decl!(
		a, abbr, address, area,
		article, aside, audio, b,
		base, bdi, bdo, blockquote,
		body, br, button, canvas,
		caption, cite, code, col,
		colgroup, data, datalist, dd,
		del, details, dfn, dialog,
		div, dl, dt, em,
		embed, fieldset, figcaption, figure,
		footer, form, h1, h2,
		h3, h4, h5, h6,
		head, header, hgroup, hr,
		html, i, iframe, img,
		input, ins, kbd, label,
		legend, li, link, main,
		map, mark, meta, meter,
		nav, noframes, noscript, object,
		ol, optgroup, option, output,
		p, param, picture, pre,
		progress, q, rp, rt,
		rtc, ruby, s, samp,
		script, section, select, slot,
		small, source, span, strong,
		style, sub, summary, sup,
		table, tbody, td, template,
		textarea, tfoot, th, thead,
		time, title, tr, track,
		u, ul, var, video,
		wbr,
	);

	impl From<Tag> for XmlElement {
		fn from(x: Tag) -> Self {
			XmlElement::Html(x)
		}
	}
}

pub mod svg {
	use super::{XmlElement, EventHandler};
	use crate::vdom::Element;
	use strum_macros::AsStaticStr;
	use std::collections::HashMap;

	macro_rules! __decl {
		($($name: ident),+$(,)*) => {
			#[allow(non_camel_case_types, dead_code)]
			#[derive(Debug, AsStaticStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
			pub enum Tag {
				$($name,)+
			}

			$(
				#[allow(dead_code, non_snake_case)]
				pub fn $name(
					children: Vec<Element>,
					attributes: HashMap<String, String>,
					event_handlers: Vec<EventHandler>,
				) -> Element {
					Element::new(XmlElement::Svg(Tag::$name), children, attributes, event_handlers)
				}
			)+
		};
	}

	__decl!(
		a, altGlyph, altGlyphDef, altGlyphItem,
		animate, animateColor, animateMotion, animateTransform,
		circle, clipPath, color_profile, cursor,
		defs, desc, discard, ellipse,
		feBlend, feColorMatrix, feComponentTransfer, feComposite,
		feConvolveMatrix, feDiffuseLighting, feDisplacementMap, feDistantLight,
		feDropShadow, feFlood, feFuncA, feFuncB,
		feFuncG, feFuncR, feGaussianBlur, feImage,
		feMerge, feMergeNode, feMorphology, feOffset,
		fePointLight, feSpecularLighting, feSpotLight, feTile,
		feTurbulence, filter, font, font_face,
		font_face_format, font_face_name, font_face_src, font_face_uri,
		foreignObject, g, glyph, glyphRef,
		hatch, hatchpath, hkern, image,
		line, linearGradient, marker, mask,
		mesh, meshgradient, meshpatch, meshrow,
		metadata, missing_glyph, mpath, path,
		pattern, polygon, polyline, radialGradient,
		rect, script, set, solidcolor,
		stop, style, svg, switch,
		symbol, text, textPath, title,
		tref, tspan, unknown, r#use,
		view, vkern,
	);

	impl From<Tag> for XmlElement {
		fn from(x: Tag) -> Self {
			XmlElement::Svg(x)
		}
	}
}
