use core::{fmt::Write, slice};
use embedded_graphics::{
	geometry::Point,
	mono_font::{ascii::FONT_9X15, *},
	pixelcolor::{self, Bgr888},
	prelude::*,
	text::*,
	Drawable,
};
use embedded_graphics_framebuf::FrameBuf;
use heapless::String;
use uefi::{proto::console::gop::GraphicsOutput, table::boot::*};

const NR_COLS: usize = 80;
const NR_LINES: usize = 30;
const LINE_SPACING: u32 = 20;

const EMPTY_STR: heapless::String<80> = String::<NR_COLS>::new();
const NORMAL_TEXT_STYLE: MonoTextStyle<'static, Bgr888> =
	MonoTextStyle::new(&FONT_9X15, Bgr888::WHITE);

/// This struct takes an instance of the framebuffer, a heapless String to write Text into,
/// a point on the display on which to write and the style in which it writes on the screen
pub struct FramebufWriter {
	framebuffer: FrameBuf<Bgr888, &'static mut [Bgr888]>,
	text: [String<NR_COLS>; NR_LINES],
	cursor: (usize, usize), // (line, column)
	first_line: isize,
}

impl FramebufWriter {
	pub fn new(framebuffer: FrameBuf<Bgr888, &'static mut [Bgr888]>) -> FramebufWriter {
		FramebufWriter {
			framebuffer,
			text: [EMPTY_STR; NR_LINES],
			cursor: (0, 0),
			first_line: -(NR_LINES as isize),
		}
	}

	fn write_out(&mut self) {
		self.framebuffer.clear(Bgr888::BLACK).unwrap();
		let mut p = Point::new(15, 15);
		let start = if self.first_line <= 0 {
			0
		} else {
			self.first_line as usize
		};
		for line_nr in (start..NR_LINES).chain(0..start) {
			Text::new(&self.text[line_nr], p, NORMAL_TEXT_STYLE)
				.draw(&mut self.framebuffer)
				.unwrap();
			p += Size::new(0, LINE_SPACING);
		}
	}

	fn new_line(&mut self) {
		self.cursor = (((self.cursor.0 + 1) % NR_LINES), 0);
		self.text[self.cursor.0].clear();
		self.first_line += 1;
		if self.first_line >= (NR_LINES as isize) {
			self.first_line %= NR_LINES as isize;
		}
	}
}

impl Write for FramebufWriter {
	fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
		for c in s.chars() {
			match c {
				'\n' => self.new_line(),
				c => {
					if self.cursor.1 >= NR_LINES {
						self.new_line();
					}

					self.text[self.cursor.0].push(c).unwrap();
					self.cursor.1 += 1;
				}
			}
		}
		self.write_out();
		Ok(())
	}
}

/// This function takes the Graphics Output Protocol (GOP), extracts the raw pointer of the framebuffer
/// and returns a wrapped instance of the framebuffer
pub fn get_framebuffer(
	gop: &mut ScopedProtocol<'_, GraphicsOutput>,
) -> FrameBuf<Bgr888, &'static mut [Bgr888]> {
	let gop_mode = gop.current_mode_info();
	let (width, height) = gop_mode.resolution();
	let framebuf_ptr = gop.frame_buffer().as_mut_ptr();
	let data = unsafe {
		slice::from_raw_parts_mut(framebuf_ptr as *mut pixelcolor::Bgr888, width * height)
	};
	FrameBuf::new(data, width, height)
}
