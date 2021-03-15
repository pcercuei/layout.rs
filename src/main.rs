// SPDX-License-Identifier: LGPL-2.1-only
//
// Copyright (C) 2021 Paul Cercueil <paul@crapouillou.net>
//
// Based on the awesome layout library by Andrew Richards
// (https://github.com/randrew/layout)

mod lib;
use lib::*;

pub struct Widget {
	pub contain_flags: u8,
	pub behave_flags: u8,

	pub margins: LayVec4,
	pub size: LayVec2,
	pub name: &'static str,

	rect: LayVec4,
	children: Vec<Box<Widget>>,
}


impl Widget
{
	pub fn new() -> Box<Widget>
	{
		Box::new(Widget {
			contain_flags: 0,
			behave_flags: 0,
			margins: [0i16; 4] as LayVec4,
			size: [0i16; 2] as LayVec2,
			rect: [0i16; 4] as LayVec4,
			children: Vec::new(),
			name: "",
		})
	}

	pub fn print(&self)
	{
		println!("{}: Position: {}x{}, Size: {}x{}", self.name,
				 self.rect[0], self.rect[1], self.rect[2], self.rect[3]);

		for each in &self.children {
			each.print();
		}
	}

	pub fn insert(&mut self, child: Box<Widget>) -> usize
	{
		self.children.push(child);

		return self.children.len();
	}

	pub fn append(&mut self, child: Box<Widget>, index: usize)
	{
		self.children.insert(index, child);
	}
}

impl<'a> LayItem<'a> for Widget {
	fn contain_flags(&self) -> u8
	{
		return self.contain_flags;
	}

	fn behave_flags(&self) -> u8
	{
		return self.behave_flags;
	}

	fn size(&self) -> LayVec2
	{
		return self.size;
	}

	fn margins(&self) -> LayVec4
	{
		return self.margins;
	}

	fn set_contain_flags(&mut self, contain: u8)
	{
		self.contain_flags = contain;
	}

	fn set_behave_flags(&mut self, behave: u8)
	{
		self.behave_flags = behave;
	}

	fn set_size(&mut self, size: LayVec2)
	{
		self.size = size;
	}

	fn set_margins(&mut self, margins: LayVec4)
	{
		self.margins = margins;
	}

	fn children(&self) -> &Vec<Box<Self>>
	{
		return &self.children;
	}

	fn children_mut(&mut self) -> &mut Vec<Box<Self>>
	{
		return &mut self.children;
	}

	fn rect(&self) -> LayVec4
	{
		self.rect
	}

	fn set_rect(&mut self, rect: LayVec4)
	{
		self.rect = rect;
	}
}

pub fn main()
{
	let mut top = Widget::new();

	top.size = [640, 480];
	top.contain_flags = LAY_COLUMN;
	top.name = "top";

	let mut topbar = Widget::new();
	topbar.contain_flags = LAY_ROW | LAY_MIDDLE | LAY_WRAP;
	topbar.behave_flags = LAY_HFILL;
	topbar.name = "topbar";

	let mut center = Widget::new();
	center.behave_flags = LAY_FILL;
	center.name = "center";

	let mut bottombar = Widget::new();
	bottombar.contain_flags = LAY_ROW | LAY_START;
	bottombar.behave_flags = LAY_HFILL;
	bottombar.name = "bottombar";

	/* Add stuff in the middle of the top bar */
	let mut widget1 = Widget::new();
	widget1.size = [32, 32];
	widget1.name = "widget1";
	topbar.insert(widget1);

	/* Add stuff on the left of the bottom bar */
	let mut widget2 = Widget::new();
	widget2.behave_flags = LAY_BREAK;
	widget2.size = [32, 32];
	widget2.name = "widget2";
	topbar.insert(widget2);

	let mut widget3 = Widget::new();
	widget3.size = [64, 64];
	widget3.name = "widget3";
	topbar.insert(widget3);

	/* Add the topbar, center, bottombar */
	top.insert(topbar);
	top.insert(center);
	top.insert(bottombar);

	/* Run the thing! */
	top.run();

	/* Read back the settings */
	top.print();
}

// vim: set noexpandtab ts=4 sw=4:
