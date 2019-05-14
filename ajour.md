## CSS
* Hashmap of string selector and rule
* `&` in selectors replaced by `styled1243246`
* Color(r, g, b, a)
* enum Unit {
	Px(f32), // no point specifying any other absolute length since they're all multiples of px
	Em(f32),
	Ex(f32),
	Ch(f32),
	Rem(f32),
	Vw(f32),
	Vh(f32),
	Vmin(f32),
	Vmax(f32),
	Percent(f32),
	Calc(String),
}

### Rules examples
* Display::Flex
	* Maybe Display::Flex {
		direction: Direction::Row,
		wrap: Wrap::NoWrap,
		justify: Justify::Start/End/Center/SpaceAround/SpaceBetween,
		align_items: AlignItems::Stretch/Start/End/Center/Baseline,
		align_content: AlignContent::Stretch/Start/End/Center/SpaceBetween/SpaceAround,
		..Default::default(),
	}
	* Maybe Display::Flex::new()
		.direction(Direction::Row)
		.wrap(Wrap::NoWrap)
		.justify(Justify::Start)
		etc
* Background {
	color: Color(r, g, b, a),
	position: (Px(10), Percent(50)),
}
* Text {
	color: Color(r, g, b, a),
	size: Unit::Px(10),
	weight, etc
}
