use crate::{FnCmp, StateRc};
use std::collections::HashMap;
use stdweb::{
	traits::*,
	web::{document, Element, Node},
};
use maplit::*;

macro_rules! __primitive {
	($name: ident) => {
		#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used)]
		pub fn $name(
			state_rc: &StateRc,
			children: &[FnCmp],
			attributes: &HashMap<String, String>,
			attach_events: impl Fn(&Element),
		) -> Element {
			let element = document().create_element(stringify!($name)).unwrap();

			for child in children {
				element.append_child(&child.0(&state_rc));
			}

			for (name, value) in attributes.iter() {
				element.set_attribute(name, value).unwrap();
			}

			attach_events(&element);

			element
		}
	};
}

__primitive!(a);__primitive!(abbr);__primitive!(address);__primitive!(area);
__primitive!(article);__primitive!(aside);__primitive!(audio);__primitive!(b);
__primitive!(base);__primitive!(bdi);__primitive!(bdo);__primitive!(big);
__primitive!(blockquote);__primitive!(body);__primitive!(br);__primitive!(button);
__primitive!(canvas);__primitive!(caption);__primitive!(circle);__primitive!(cite);
__primitive!(clipPath);__primitive!(code);__primitive!(col);__primitive!(colgroup);
__primitive!(data);__primitive!(datalist);__primitive!(dd);__primitive!(defs);
__primitive!(del);__primitive!(details);__primitive!(dfn);__primitive!(dialog);
__primitive!(div);__primitive!(dl);__primitive!(dt);__primitive!(ellipse);
__primitive!(em);__primitive!(embed);__primitive!(fieldset);__primitive!(figcaption);
__primitive!(figure);__primitive!(footer);__primitive!(foreignObject);__primitive!(form);
__primitive!(g);__primitive!(h1);__primitive!(h2);__primitive!(h3);
__primitive!(h4);__primitive!(h5);__primitive!(h6);__primitive!(head);
__primitive!(header);__primitive!(hgroup);__primitive!(hr);__primitive!(html);
__primitive!(i);__primitive!(iframe);__primitive!(image);__primitive!(img);
__primitive!(input);__primitive!(ins);__primitive!(kbd);__primitive!(keygen);
__primitive!(label);__primitive!(legend);__primitive!(li);__primitive!(line);
__primitive!(linearGradient);__primitive!(link);__primitive!(main);__primitive!(map);
__primitive!(mark);__primitive!(marquee);__primitive!(mask);__primitive!(menu);
__primitive!(menuitem);__primitive!(meta);__primitive!(meter);__primitive!(nav);
__primitive!(noscript);__primitive!(object);__primitive!(ol);__primitive!(optgroup);
__primitive!(option);__primitive!(output);__primitive!(p);__primitive!(param);
__primitive!(path);__primitive!(pattern);__primitive!(picture);__primitive!(polygon);
__primitive!(polyline);__primitive!(pre);__primitive!(progress);__primitive!(prototype);
__primitive!(q);__primitive!(radialGradient);__primitive!(rect);__primitive!(rp);
__primitive!(rt);__primitive!(ruby);__primitive!(s);__primitive!(samp);
__primitive!(script);__primitive!(section);__primitive!(select);__primitive!(small);
__primitive!(source);__primitive!(span);__primitive!(stop);__primitive!(strong);
__primitive!(style);__primitive!(sub);__primitive!(summary);__primitive!(sup);
__primitive!(svg);__primitive!(table);__primitive!(tbody);__primitive!(td);
__primitive!(text);__primitive!(textarea);__primitive!(tfoot);__primitive!(th);
__primitive!(thead);__primitive!(time);__primitive!(title);__primitive!(tr);
__primitive!(track);__primitive!(tspan);__primitive!(u);__primitive!(ul);
__primitive!(var);__primitive!(video);__primitive!(wbr);

impl From<String> for FnCmp {
	fn from(s: String) -> Self {
		FnCmp(Box::new(move |_| {
			let p = document().create_element("span").unwrap();
			p.set_text_content(&s);
			p
		}))
	}
}

impl From<&str> for FnCmp {
	fn from(s: &str) -> Self {
		let owned = s.to_owned();
		FnCmp(Box::new(move |_| {
			let p = document().create_element("span").unwrap();
			p.set_text_content(&owned);
			p
		}))
	}
}

macro_rules! children {
	($($e: expr),+$(,)*) => {
		&[$($e.into(),)+]
	};
}

macro_rules! attrs {
	() => {
		&hashmap![]
	};
	($($e: expr),+$(,)*) => {
		&hashmap![$($e.into(),)+]
	};
}
