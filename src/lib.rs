// SPDX-License-Identifier: LGPL-2.1-only
//
// Copyright (C) 2021 Paul Cercueil <paul@crapouillou.net>
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


pub trait LayItem<'a> where Self: 'a {
	fn contain_flags(&self) -> u8;
	fn behave_flags(&self) -> u8;
	fn size(&self) -> LayVec2;
	fn margins(&self) -> LayVec4;

	fn set_contain_flags(&mut self, contain: u8);
	fn set_behave_flags(&mut self, behave: u8);
	fn set_size(&mut self, size: LayVec2);
	fn set_margins(&mut self, margins: LayVec4);

	fn children(&self) -> &Vec<Box<Self>>;
	fn children_mut(&mut self) -> &mut Vec<Box<Self>>;

	fn rect(&self) -> LayVec4;
	fn set_rect(&mut self, rect: LayVec4);

	fn run(&mut self)
	{
		self.calc_size(0);
		self.arrange(0);
		self.calc_size(1);
		self.arrange(1);
	}

	fn calc_stacked_size(&self, dim: usize) -> i16
	{
		let mut need_size: i16 = 0;

		for each in self.children() {
			let rect = each.rect();
			let margins = each.margins();

			need_size += rect[dim] + rect[dim + 2] + margins[dim + 2];
		}

		return need_size;
	}

	fn calc_overlayed_size(&self, dim: usize) -> i16
	{
		let mut need_size: i16 = 0;

		for each in self.children() {
			let rect = each.rect();
			let margins = each.margins();

			let child_size: i16 = rect[dim] + rect[dim + 2] + margins[dim + 2];

			need_size = i16::max(need_size, child_size);
		}

		return need_size;
	}

	fn calc_wrapped_overlayed_size(&self) -> i16
	{
		let mut need_size: i16 = 0;
		let mut need_size2: i16 = 0;

		for each in self.children() {
			let rect = each.rect();
			let margins = each.margins();

			if each.behave_flags() & LAY_BREAK != 0 {
				need_size2 += need_size;
				need_size = 0;
			}

			let child_size: i16 = rect[1] + rect[3] + margins[3];
			need_size = i16::max(need_size, child_size);
		}

		return need_size2 + need_size;
	}

	fn calc_wrapped_stacked_size(&self) -> i16
	{
		let mut need_size: i16 = 0;
		let mut need_size2: i16 = 0;

		for each in self.children() {
			let rect = each.rect();
			let margins = each.margins();

			if each.behave_flags() & LAY_BREAK != 0 {
				need_size2 = i16::max(need_size2, need_size);
				need_size = 0;
			}

			need_size += rect[0] + rect[2] + margins[2];
		}

		return i16::max(need_size2, need_size);
	}

	fn calc_size(&mut self, dim: usize)
	{
		for each in self.children_mut() {
			each.calc_size(dim);
		}

		let mut rect = self.rect();
		let margins = self.margins();

		rect[dim] = margins[dim];

		if self.size()[dim] != 0 {
			rect[dim + 2] = self.size()[dim];
			self.set_rect(rect);
			return;
		}

		let cal_size: i16;

		match self.contain_flags() & LAY_LAYOUT_FLAGS {
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
				if (self.contain_flags() & 1) as usize == dim {
					cal_size = self.calc_stacked_size(dim);
				} else {
					cal_size = self.calc_overlayed_size(dim);
				}
			}

			_ => {
				cal_size = self.calc_overlayed_size(dim);
			}
		}

		rect[dim + 2] = cal_size;
		self.set_rect(rect);
	}

	fn arrange_stacked(&mut self, dim: usize, wrap: bool)
	{
		let nb_children: usize = self.children().len();
		let rect = self.rect();
		let max_x2: f32 = (rect[dim] + rect[dim + 2]) as f32;

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
				let child = &mut self.children_mut()[idx];
				let child_rect = child.rect();
				let child_margins = child.margins();
				let mut extend: i16 = used;

				if (child.behave_flags() >> dim) & LAY_HFILL == LAY_HFILL {
					count += 1;
					extend += child_rect[dim] + child_margins[dim + 2];
				} else {
					if child.size()[dim] == 0 {
						squeezed_count += 1;
					}

					extend += child_rect[dim] + child_rect[dim + 2] + child_margins[dim + 2];
				}

				if wrap && total > 0 &&
					(extend > rect[dim + 2] || child.behave_flags() & LAY_BREAK == LAY_BREAK) {
						last = idx;
						hardbreak = child.behave_flags() & LAY_BREAK == LAY_BREAK;
						child.set_behave_flags(child.behave_flags() | LAY_BREAK);
						break;
				}

				used = extend;
				total += 1;
				idx += 1;
			}

			let extra_space: f32 = (rect[dim + 2] - used) as f32;
			let mut filler: f32 = 0.0;
			let mut spacer: f32 = 0.0;
			let mut extra_margin: f32 = 0.0;
			let mut eater: f32 = 0.0;

			if extra_space > 0.0 {
				if count > 0 {
					filler = extra_space / count as f32;
				} else if total > 0 {
					match self.contain_flags() & LAY_JUSTIFY {
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
			let mut x: f32 = rect[dim] as f32;
			let mut x1: f32;

			idx = start;
			while idx < last {
				let ix0: i16;
				let ix1: i16;
				let child = &mut self.children_mut()[idx];
				let child_margins = child.margins();
				let mut child_rect = child.rect();

				x += child_rect[dim] as f32 + extra_margin;

				if (child.behave_flags() >> dim) & LAY_HFILL == LAY_HFILL {
					x1 = x + filler;
				} else if child.size()[dim] != 0 {
					x1 = x + child_rect[dim + 2] as f32;
				} else {
					x1 = x + f32::max(0.0, child_rect[dim + 2] as f32 + eater);
				}

				ix0 = x as i16;

				if wrap {
					ix1 = f32::min(max_x2 - child_margins[dim + 2] as f32, x1) as i16;
				} else {
					ix1 = x1 as i16;
				}

				child_rect[dim] = ix0;
				child_rect[dim + 2] = ix1 - ix0;

				child.set_rect(child_rect);

				x = x1 + child_margins[dim + 2] as f32;

				extra_margin = spacer;
				idx += 1;
			}

			start = last;
		}
	}

	fn arrange_overlay(&mut self, dim: usize)
	{
		let rect = self.rect();

		for each in self.children_mut() {
			let mut child_rect = each.rect();
			let child_margins = each.margins();
			let flags: u8 = (each.behave_flags() >> dim) & LAY_HFILL;
			let space: i16 = rect[dim + 2];

			match flags {
				LAY_HCENTER => {
					child_rect[dim] += (space - child_rect[dim + 2]) / 2 - child_margins[dim + 2];
				}

				LAY_RIGHT => {
					child_rect[dim] += space - child_rect[dim + 2]
						- child_margins[dim] - child_margins[dim + 2];
				}

				LAY_HFILL => {
					child_rect[dim + 2] = i16::max(0, space - child_rect[dim]
												  - child_margins[dim + 2]);
				}

				_ => {}
			}

			child_rect[dim] += rect[dim];

			each.set_rect(child_rect);
		}
	}

	fn arrange_overlay_squeezed(&mut self, dim: usize, offset: i16, space: i16)
	{
		let mut rect = self.rect();
		let margins = self.margins();
		let min_size = i16::max(0, space - rect[dim] - margins[dim + 2]);
		let flags: u8 = (self.behave_flags() >> dim) & LAY_HFILL;

		match flags {
			LAY_HCENTER => {
				rect[dim + 2] = i16::min(rect[dim + 2], min_size);
				rect[dim] += (space - rect[dim + 2]) / 2 - margins[dim + 2];
			}

			LAY_RIGHT => {
				rect[dim + 2] = i16::min(rect[dim + 2], min_size);
				rect[dim] = space - rect[dim + 2] - margins[dim + 2];
			}

			LAY_HFILL => {
				rect[dim + 2] = min_size;
			}

			_ => {
				rect[dim + 2] = i16::min(rect[dim + 2], min_size);
			}
		}

		rect[dim] += offset;
		self.set_rect(rect);
	}

	fn arrange_overlay_squeezed_range(&mut self, dim: usize, mut start: usize,
									  end: usize, offset: i16, space: i16)
	{
		while start != end {
			let child = &mut self.children_mut()[start];

			child.arrange_overlay_squeezed(dim, offset, space);

			start += 1;
		}
	}

	fn arrange_wrapped_overlay_squeezed(&mut self, dim: usize) -> i16
	{
		let nb_children = self.children().len();
		let mut offset: i16 = self.rect()[dim];
		let mut need_size: i16 = 0;
		let mut start: usize = 0;
		let mut idx: usize = 0;

		while idx < nb_children {
			let child = &mut self.children_mut()[idx];
			let child_rect = child.rect();
			let child_size: i16 = child_rect[dim] + child_rect[dim + 2] + child.margins()[dim + 2];

			if child.behave_flags() & LAY_BREAK != 0 {
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
		let mut rect = self.rect();

		match self.contain_flags() & LAY_LAYOUT_FLAGS {
			a if a == LAY_COLUMN | LAY_WRAP => {
				if dim > 0 {
					self.arrange_stacked(1, true);

					let offset: i16 = self.arrange_wrapped_overlay_squeezed(0);
					rect[2] = offset - rect[0];
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
				if (self.contain_flags() & 1) as usize == dim {
					self.arrange_stacked(dim, false);
				} else {
					for each in self.children_mut() {
						each.arrange_overlay_squeezed(dim, rect[dim], rect[dim + 2]);
					}
				}
			}

			_ => {
				self.arrange_overlay(dim);
			}
		}

		self.set_rect(rect);

		for each in self.children_mut() {
			each.arrange(dim);
		}
	}
}

// vim: set noexpandtab ts=4 sw=4:
