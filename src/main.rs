// SPDX-License-Identifier: LGPL-2.1-only
//
// Copyright (C) 2021 Paul Cercueil <paul@crapouillou.net>
//
// Based on the awesome layout library by Andrew Richards
// (https://github.com/randrew/layout)

mod lib;
use lib::*;

struct Widget<'a> {
	pub base: Box<BaseItem<'a>>,
}

impl<'a> Widget<'a> {
	pub fn new(name: &'static str,
			   contain_flags: u8,
			   behave_flags: u8,
			   size: LayVec2) -> Box<Widget<'a>>
	{
		Box::new(Self {
			base: BaseItem::new(name, contain_flags, behave_flags, size)
		})
	}
}

impl<'a> LayoutItem<'a> for Widget<'a>
{
	fn base(&self) -> &BaseItem<'a> { &self.base }
	fn base_mut(&mut self) -> &mut BaseItem<'a> { &mut self.base }

	fn handle_click(&mut self, _pos: LayVec2) -> bool { false }
	fn handle_key(&mut self, _key: u32, _event: u32) -> bool { false }
}

pub fn main()
{
	let mut top = Widget::new("top", LAY_COLUMN, 0, [640, 480]);
	let mut topbar = Widget::new("topbar", LAY_ROW | LAY_MIDDLE | LAY_WRAP,
								 LAY_HFILL, [0, 0]);
	let center = Widget::new("center", 0, LAY_FILL, [0, 0]);
	let bottombar = Widget::new("bottombar", LAY_ROW | LAY_START, LAY_HFILL, [0, 0]);

	/* Add stuff in the middle of the top bar */
	let widget1 = Widget::new("widget1", 0, 0, [32, 32]);
	topbar.base.insert(widget1);

	/* Add stuff on the left of the bottom bar */
	let widget2 = Widget::new("widget2", 0, LAY_BREAK, [32, 32]);
	topbar.base.insert(widget2);

	let widget3 = Widget::new("widget3", 0, 0, [64, 64]);
	topbar.base.insert(widget3);

	/* Add the topbar, center, bottombar */
	top.base.insert(topbar);
	top.base.insert(center);
	top.base.insert(bottombar);

	/* Run the thing! */
	top.base.run();

	/* Read back the settings */
	top.base.print();
}

// vim: set noexpandtab ts=4 sw=4:
