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

/// This struct takes an instance of the framebuffer, a heapless String to write Text into,
/// a point on the display on which to write and the style in which it writes on the screen
pub struct FramebufWriter {
	framebuffer: FrameBuf<Bgr888, &'static mut [Bgr888]>,
	text: String<80>,
	point: Point,
	style: MonoTextStyle<'static, Bgr888>,
}

impl FramebufWriter {
	pub fn new(framebuffer: FrameBuf<Bgr888, &'static mut [Bgr888]>) -> FramebufWriter {
		FramebufWriter {
			framebuffer,
			text: heapless::String::<80>::new(),
			point: Point::new(15, 15),
			style: MonoTextStyle::new(&FONT_9X15, Bgr888::WHITE),
		}
	}

	// writes to screen, can support one numerical argument and sets Point to the next line
	pub fn write(&mut self, line: &str, arg1: Option<usize>) {
		self.text.clear();
		match arg1 {
			Some(arg) => write!(&mut self.text, "{}: {:#x?}", line, arg).unwrap(),
			None => write!(&mut self.text, "{}", line).unwrap(),
		}
		Text::new(&self.text, self.point, self.style)
			.draw(&mut self.framebuffer)
			.unwrap();
		self.point += Size::new(0, 20);
	}
}

// impl Write for FramebufWriter {
// 	fn write(&mut self, buf: &[u8]) -> Result<usize> {

// 	}
//     fn flush(&mut self) -> Result<()> {

// 	}
// }

/// This function takes the Graphics Output Protocol (GOP), extracts the raw pointer of the framebuffer
/// and returns a wrapped instance of the framebuffer
pub fn get_framebuffer(
	gop: &mut ScopedProtocol<'_, GraphicsOutput>,
) -> FrameBuf<Bgr888, &'static mut [Bgr888]> {
	let gop_mode = gop.current_mode_info();
	let (width, height) = gop_mode.resolution();
	let mut framebuf_ptr = gop.frame_buffer().as_mut_ptr();
	let data = unsafe {
		slice::from_raw_parts_mut(framebuf_ptr as *mut pixelcolor::Bgr888, width * height)
	};
	FrameBuf::new(data, width, height)
}
