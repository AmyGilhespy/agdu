pub mod tailwind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RgbColor888 {
	pub r8: u8,
	pub g8: u8,
	pub b8: u8,
}

impl RgbColor888 {
	#[must_use]
	#[allow(clippy::missing_panics_doc)]
	pub fn from_rgb24(rgb24: u32) -> Self {
		Self {
			r8: u8::try_from(rgb24 >> 16 & 0xff).unwrap(),
			g8: u8::try_from(rgb24 >> 8 & 0xff).unwrap(),
			b8: u8::try_from(rgb24 & 0xff).unwrap(),
		}
	}
}

#[cfg(feature = "godot")]
impl From<RgbColor888> for godot::classes::class_macros::private::virtuals::Os::Color {
	fn from(value: RgbColor888) -> Self {
		use godot::classes::class_macros::private::virtuals::Os::Color;
		Color::from_rgba8(value.r8, value.g8, value.b8, 255)
	}
}

#[cfg(feature = "godot")]
impl From<godot::classes::class_macros::private::virtuals::Os::Color> for RgbColor888 {
	fn from(value: godot::classes::class_macros::private::virtuals::Os::Color) -> Self {
		RgbColor888 {
			r8: value.r8(),
			g8: value.g8(),
			b8: value.b8(),
		}
	}
}
