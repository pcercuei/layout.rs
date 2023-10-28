// SPDX-License-Identifier: LGPL-2.1-only
//
// Copyright (C) 2021-2023 Paul Cercueil <paul@crapouillou.net>
//
// Based on the awesome layout library by Andrew Richards
// (https://github.com/randrew/layout)

/* Contain flags */
pub const LAY_ROW:			u8 = 0x02;
pub const LAY_COLUMN:		u8 = 0x03;
pub const LAY_LAYOUT:		u8 = 0x00;
pub const LAY_NOWRAP:		u8 = 0x00;
pub const LAY_WRAP:			u8 = 0x04;
pub const LAY_LAYOUT_FLAGS:	u8 = 0x07;
pub const LAY_START:		u8 = 0x08;
pub const LAY_MIDDLE:		u8 = 0x00;
pub const LAY_END:			u8 = 0x10;
pub const LAY_JUSTIFY:		u8 = LAY_START | LAY_END;

/* Behave flags */
pub const LAY_LEFT:			u8 = 0x01;
pub const LAY_TOP:			u8 = 0x02;
pub const LAY_RIGHT:		u8 = 0x04;
pub const LAY_BOTTOM:		u8 = 0x08;
pub const LAY_HFILL:		u8 = LAY_LEFT | LAY_RIGHT;
pub const LAY_VFILL:		u8 = LAY_TOP | LAY_BOTTOM;
pub const LAY_FILL:			u8 = LAY_HFILL | LAY_VFILL;
pub const LAY_HCENTER:		u8 = 0x00;
pub const LAY_VCENTER:		u8 = 0x00;
pub const LAY_CENTER:		u8 = LAY_HCENTER | LAY_VCENTER;
pub const LAY_BREAK:		u8 = 0x10;


pub type LayVec2 = [i16; 2];
pub type LayVec4 = [i16; 4];

pub struct BaseItem<'a> {
	pub contain_flags: u8,
	pub behave_flags: u8,

	pub margins: LayVec4,
	pub size: LayVec2,
	pub name: &'a str,

	rect: LayVec4,
	children: Vec<Box<dyn LayoutItem<'a>>>,
}

pub trait LayoutItem<'a> where Self: 'a {
	fn base_mut(&mut self) -> &mut BaseItem<'a>;
	fn base(&self) -> &BaseItem<'a>;

	fn handle_click(&mut self, pos: LayVec2) -> bool;
	fn handle_key(&mut self, key: u32, event: u32) -> bool;
}

impl<'a> BaseItem<'a>
{
	pub fn new(name: &'a str,
			   contain_flags: u8,
			   behave_flags: u8,
			   size: LayVec2) -> Box<BaseItem<'a>>
	{
		Box::new(BaseItem {
			contain_flags: contain_flags,
			behave_flags: behave_flags,
			margins: [0i16; 4] as LayVec4,
			size: size,
			rect: [0i16; 4] as LayVec4,
			children: Vec::new(),
			name: name,
		})
	}

	pub fn print(&self)
	{
		println!("{}: Position: {}x{}, Size: {}x{}", self.name,
				 self.rect[0], self.rect[1], self.rect[2], self.rect[3]);

		for each in &self.children {
			each.base().print();
		}
	}

	pub fn insert(&mut self, child: Box<dyn LayoutItem<'a>>) -> usize
	{
		self.children.push(child);

		return self.children.len();
	}

	pub fn append(&mut self, child: Box<dyn LayoutItem<'a>>, index: usize)
	{
		self.children.insert(index, child);
	}

	pub fn position(&self) -> LayVec4
	{
		return self.rect;
	}

	pub fn click(&mut self, pos: LayVec2) -> bool
	{
		for each in &mut self.children {
			if each.base_mut().click(pos) {
				return true;
			}
		}

		if pos[0] >= self.rect[0]
			&& pos[0] < self.rect[0] + self.rect[2]
			&& pos[1] >= self.rect[1]
			&& pos[1] < self.rect[1] + self.rect[3] {
			return self.handle_click([pos[0] - self.rect[0], pos[1] - self.rect[1]]);
		}

		return false;
	}

	pub fn key(&mut self, key: u32, event: u32) -> bool
	{
		for each in &mut self.children {
			if each.base_mut().key(key, event) {
				return true;
			}
		}

		return self.handle_key(key, event);
	}

	pub fn run(&mut self)
	{
		self.calc_size(0);
		self.arrange(0);
		self.calc_size(1);
		self.arrange(1);
	}

	fn calc_stacked_size(&self, dim: usize) -> i16
	{
		let mut need_size: i16 = 0;

		for each in &self.children {
			let base = each.base();

			need_size += base.rect[dim] + base.rect[dim + 2] + base.margins[dim + 2];
		}

		return need_size;
	}

	fn calc_overlayed_size(&self, dim: usize) -> i16
	{
		let mut need_size: i16 = 0;

		for each in &self.children {
			let base = each.base();
			let child_size: i16 = base.rect[dim] + base.rect[dim + 2] + base.margins[dim + 2];

			need_size = i16::max(need_size, child_size);
		}

		return need_size;
	}

	fn calc_wrapped_overlayed_size(&self) -> i16
	{
		let mut need_size: i16 = 0;
		let mut need_size2: i16 = 0;

		for each in &self.children {
			let base = each.base();

			if base.behave_flags & LAY_BREAK != 0 {
				need_size2 += need_size;
				need_size = 0;
			}

			let child_size: i16 = base.rect[1] + base.rect[3] + base.margins[3];
			need_size = i16::max(need_size, child_size);
		}

		return need_size2 + need_size;
	}

	fn calc_wrapped_stacked_size(&self) -> i16
	{
		let mut need_size: i16 = 0;
		let mut need_size2: i16 = 0;

		for each in &self.children {
			let base = each.base();

			if base.behave_flags & LAY_BREAK != 0 {
				need_size2 = i16::max(need_size2, need_size);
				need_size = 0;
			}

			need_size += base.rect[0] + base.rect[2] + base.margins[2];
		}

		return i16::max(need_size2, need_size);
	}

	fn calc_size(&mut self, dim: usize)
	{
		for each in &mut self.children {
			each.base_mut().calc_size(dim);
		}

		self.rect[dim] = self.margins[dim];

		if self.size[dim] != 0 {
			self.rect[dim + 2] = self.size[dim];
			return;
		}

		let cal_size: i16;

		match self.contain_flags & LAY_LAYOUT_FLAGS {
			a if a == LAY_COLUMN | LAY_WRAP => {
				/* Flex model */
				if dim > 0 {
					cal_size = self.calc_stacked_size(1);
				} else {
					cal_size = self.calc_overlayed_size(0);
				}
			}

			a if a == LAY_ROW | LAY_WRAP => {
				/* Flex model */
				if dim == 0 {
					cal_size = self.calc_wrapped_stacked_size();
				} else {
					cal_size = self.calc_wrapped_overlayed_size();
				}
			}

			LAY_COLUMN | LAY_ROW => {
				if (self.contain_flags & 1) as usize == dim {
					cal_size = self.calc_stacked_size(dim);
				} else {
					cal_size = self.calc_overlayed_size(dim);
				}
			}

			_ => {
				cal_size = self.calc_overlayed_size(dim);
			}
		}

		self.rect[dim + 2] = cal_size;
	}

	fn arrange_stacked(&mut self, dim: usize, wrap: bool)
	{
		let nb_children: usize = self.children.len();
		let max_x2: f32 = (self.rect[dim] + self.rect[dim + 2]) as f32;

		let mut start: usize = 0;

		while start < nb_children {
			let mut used: i16 = 0;
			let mut count: usize = 0;
			let mut squeezed_count: usize = 0;
			let mut total: usize = 0;
			let mut hardbreak: bool = false;

			/*
			 * First pass: count items that need to be expanded,
			 * and the space that is used.
			 */
			let mut idx: usize = start;
			let mut last = nb_children;

			while idx < nb_children {
				let child = self.children[idx].base_mut();
				let mut extend: i16 = used;

				if (child.behave_flags >> dim) & LAY_HFILL == LAY_HFILL {
					count += 1;
					extend += child.rect[dim] + child.margins[dim + 2];
				} else {
					if child.size[dim] == 0 {
						squeezed_count += 1;
					}

					extend += child.rect[dim] + child.rect[dim + 2] + child.margins[dim + 2];
				}

				if wrap && total > 0 &&
					(extend > self.rect[dim + 2] || child.behave_flags & LAY_BREAK == LAY_BREAK) {
						last = idx;
						hardbreak = child.behave_flags & LAY_BREAK == LAY_BREAK;
						child.behave_flags |= LAY_BREAK;
						break;
				}

				used = extend;
				total += 1;
				idx += 1;
			}

			let extra_space: f32 = (self.rect[dim + 2] - used) as f32;
			let mut filler: f32 = 0.0;
			let mut spacer: f32 = 0.0;
			let mut extra_margin: f32 = 0.0;
			let mut eater: f32 = 0.0;

			if extra_space > 0.0 {
				if count > 0 {
					filler = extra_space / count as f32;
				} else if total > 0 {
					match self.contain_flags & LAY_JUSTIFY {
						LAY_JUSTIFY => {
							/*
							 * Justify when not wrapping or not in last line,
							 * or not manually breaking
							 */

							if !wrap || (last != nb_children && !hardbreak) {
								spacer = extra_space / (total - 1) as f32;
							}
						}

						LAY_START => {}

						LAY_END => {
							extra_margin = extra_space;
						}

						_ => {
							extra_margin = extra_space / 2.0;
						}
					}
				}
			} else if !wrap && (extra_space < 0.0) {
				eater = extra_space / squeezed_count as f32;
			}

			/* Distribute width among items */
			let mut x: f32 = self.rect[dim] as f32;
			let mut x1: f32;

			idx = start;
			while idx < last {
				let ix0: i16;
				let ix1: i16;
				let child = self.children[idx].base_mut();

				x += child.rect[dim] as f32 + extra_margin;

				if (child.behave_flags >> dim) & LAY_HFILL == LAY_HFILL {
					x1 = x + filler;
				} else if child.size[dim] != 0 {
					x1 = x + child.rect[dim + 2] as f32;
				} else {
					x1 = x + f32::max(0.0, child.rect[dim + 2] as f32 + eater);
				}

				ix0 = x as i16;

				if wrap {
					ix1 = f32::min(max_x2 - child.margins[dim + 2] as f32, x1) as i16;
				} else {
					ix1 = x1 as i16;
				}

				child.rect[dim] = ix0;
				child.rect[dim + 2] = ix1 - ix0;

				x = x1 + child.margins[dim + 2] as f32;

				extra_margin = spacer;
				idx += 1;
			}

			start = last;
		}
	}

	fn arrange_overlay(&mut self, dim: usize)
	{
		for each in &mut self.children {
			let mut base = each.base_mut();
			let flags: u8 = (base.behave_flags >> dim) & LAY_HFILL;
			let space: i16 = self.rect[dim + 2];

			match flags {
				LAY_HCENTER => {
					base.rect[dim] += (space - base.rect[dim + 2]) / 2 - base.margins[dim + 2];
				}

				LAY_RIGHT => {
					base.rect[dim] += space - base.rect[dim + 2]
						- base.margins[dim] - base.margins[dim + 2];
				}

				LAY_HFILL => {
					base.rect[dim + 2] = i16::max(0, space - base.rect[dim]
												  - base.margins[dim + 2]);
				}

				_ => {}
			}

			base.rect[dim] += self.rect[dim];
		}
	}

	fn arrange_overlay_squeezed(&mut self, dim: usize, offset: i16, space: i16)
	{
		let min_size = i16::max(0, space - self.rect[dim] - self.margins[dim + 2]);
		let flags: u8 = (self.behave_flags >> dim) & LAY_HFILL;

		match flags {
			LAY_HCENTER => {
				self.rect[dim + 2] = i16::min(self.rect[dim + 2], min_size);
				self.rect[dim] += (space - self.rect[dim + 2]) / 2 - self.margins[dim + 2];
			}

			LAY_RIGHT => {
				self.rect[dim + 2] = i16::min(self.rect[dim + 2], min_size);
				self.rect[dim] = space - self.rect[dim + 2] - self.margins[dim + 2];
			}

			LAY_HFILL => {
				self.rect[dim + 2] = min_size;
			}

			_ => {
				self.rect[dim + 2] = i16::min(self.rect[dim + 2], min_size);
			}
		}

		self.rect[dim] += offset;
	}

	fn arrange_overlay_squeezed_range(&mut self, dim: usize, mut start: usize,
									  end: usize, offset: i16, space: i16)
	{
		while start != end {
			let child = self.children[start].base_mut();

			child.arrange_overlay_squeezed(dim, offset, space);

			start += 1;
		}
	}

	fn arrange_wrapped_overlay_squeezed(&mut self, dim: usize) -> i16
	{
		let nb_children = self.children.len();
		let mut offset: i16 = self.rect[dim];
		let mut need_size: i16 = 0;
		let mut start: usize = 0;
		let mut idx: usize = 0;

		while idx < nb_children {
			let child = self.children[idx].base_mut();
			let child_size: i16 = child.rect[dim] + child.rect[dim + 2] + child.margins[dim + 2];

			if child.behave_flags & LAY_BREAK != 0 {
				self.arrange_overlay_squeezed_range(dim, start, idx, offset, need_size);
				offset += need_size;
				start = idx;
				need_size = 0;
			}

			need_size = i16::max(need_size, child_size);
			idx += 1;
		}

		self.arrange_overlay_squeezed_range(dim, start, nb_children, offset, need_size);

		return offset + need_size;
	}

	fn arrange(&mut self, dim: usize)
	{
		match self.contain_flags & LAY_LAYOUT_FLAGS {
			a if a == LAY_COLUMN | LAY_WRAP => {
				if dim > 0 {
					self.arrange_stacked(1, true);

					let offset: i16 = self.arrange_wrapped_overlay_squeezed(0);
					self.rect[2] = offset - self.rect[0];
				}
			}

			a if a == LAY_ROW | LAY_WRAP => {
				if dim == 0 {
					self.arrange_stacked(0, true);
				} else {
					self.arrange_wrapped_overlay_squeezed(1);
				}
			}

			LAY_COLUMN | LAY_ROW => {
				if (self.contain_flags & 1) as usize == dim {
					self.arrange_stacked(dim, false);
				} else {
					for each in &mut self.children {
						each.base_mut().arrange_overlay_squeezed(dim, self.rect[dim], self.rect[dim + 2]);
					}
				}
			}

			_ => {
				self.arrange_overlay(dim);
			}
		}

		for each in &mut self.children {
			each.base_mut().arrange(dim);
		}
	}
}

impl<'a> LayoutItem<'a> for BaseItem<'a>
{
	fn base(&self) -> &BaseItem<'a> { &self }
	fn base_mut(&mut self) -> &mut BaseItem<'a> { self }

	fn handle_click(&mut self, _pos: LayVec2) -> bool { false }
	fn handle_key(&mut self, _key: u32, _event: u32) -> bool { false }
}

// vim: set noexpandtab ts=4 sw=4:
