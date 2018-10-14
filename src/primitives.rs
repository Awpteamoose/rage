use crate::cmp::{FnCmp, StateRc, StateLock};
use std::collections::HashMap;
use stdweb::{
	traits::*,
	web::{document, Element},
};

macro_rules! __p {
	($name: ident) => {
		#[allow(clippy::option_unwrap_used, clippy::result_unwrap_used, dead_code, non_snake_case)]
		pub fn $name<S: Default>(
			state_lock: &StateLock<S>,
			children: &[FnCmp<S>],
			attributes: &HashMap<String, String>,
			attach_events: impl Fn(&Element),
		) -> Element {
			let element = document().create_element(stringify!($name)).unwrap();

			for child in children {
				element.append_child(&child.0(state_lock));
			}

			for (name, value) in attributes.iter() {
				element.set_attribute(name, value).unwrap();
			}

			attach_events(&element);

			element
		}
	};
}

__p!(a);__p!(abbr);__p!(address);__p!(area);
__p!(article);__p!(aside);__p!(audio);__p!(b);
__p!(base);__p!(bdi);__p!(bdo);__p!(big);
__p!(blockquote);__p!(body);__p!(br);__p!(button);
__p!(canvas);__p!(caption);__p!(circle);__p!(cite);
__p!(clipPath);__p!(code);__p!(col);__p!(colgroup);
__p!(data);__p!(datalist);__p!(dd);__p!(defs);
__p!(del);__p!(details);__p!(dfn);__p!(dialog);
__p!(div);__p!(dl);__p!(dt);__p!(ellipse);
__p!(em);__p!(embed);__p!(fieldset);__p!(figcaption);
__p!(figure);__p!(footer);__p!(foreignObject);__p!(form);
__p!(g);__p!(h1);__p!(h2);__p!(h3);
__p!(h4);__p!(h5);__p!(h6);__p!(head);
__p!(header);__p!(hgroup);__p!(hr);__p!(html);
__p!(i);__p!(iframe);__p!(image);__p!(img);
__p!(input);__p!(ins);__p!(kbd);__p!(keygen);
__p!(label);__p!(legend);__p!(li);__p!(line);
__p!(linearGradient);__p!(link);__p!(main);__p!(map);
__p!(mark);__p!(marquee);__p!(mask);__p!(menu);
__p!(menuitem);__p!(meta);__p!(meter);__p!(nav);
__p!(noscript);__p!(object);__p!(ol);__p!(optgroup);
__p!(option);__p!(output);__p!(p);__p!(param);
__p!(path);__p!(pattern);__p!(picture);__p!(polygon);
__p!(polyline);__p!(pre);__p!(progress);__p!(prototype);
__p!(q);__p!(radialGradient);__p!(rect);__p!(rp);
__p!(rt);__p!(ruby);__p!(s);__p!(samp);
__p!(script);__p!(section);__p!(select);__p!(small);
__p!(source);__p!(span);__p!(stop);__p!(strong);
__p!(style);__p!(sub);__p!(summary);__p!(sup);
__p!(svg);__p!(table);__p!(tbody);__p!(td);
__p!(text);__p!(textarea);__p!(tfoot);__p!(th);
__p!(thead);__p!(time);__p!(title);__p!(tr);
__p!(track);__p!(tspan);__p!(u);__p!(ul);
__p!(var);__p!(video);__p!(wbr);

impl<S: Default> From<String> for FnCmp<S> {
	#[allow(clippy::result_unwrap_used)]
	fn from(s: String) -> Self {
		FnCmp::new(move |_| {
			let elem = document().create_element("span").unwrap();
			elem.set_text_content(&s);
			elem
		})
	}
}

impl<S: Default> From<&str> for FnCmp<S> {
	#[allow(clippy::result_unwrap_used)]
	fn from(s: &str) -> Self {
		let owned = s.to_owned();
		FnCmp::new(move |_| {
			let elem = document().create_element("span").unwrap();
			elem.set_text_content(&owned);
			elem
		})
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
