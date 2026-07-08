//! Colors can be used to easily create color values from various methods
//!
//! Colors are a common trait of many objects in a scene. colors are stored internally as
//! noramalized vectors between 0.0 and 1.0. this module helps in creating colors from more methods
//! such as from 8 bit values or a hex values
//!
//! # Example
//! ```rust
//! use maple::color::{Color, WHITE};
//!
//! // create a color
//! let color: Color = Color::from_normalized(1.0, 1.0, 1.0, 1.0);
//!
//! // or use some predefined constants
//! let color: Color = WHITE;
//! ```

use glam::{self as math, Vec3};

/// represents a linear color with rgba
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    /// red component
    pub r: f32,
    /// green component
    pub g: f32,
    /// blue component
    pub b: f32,
    /// alpha component
    pub a: f32,
}

impl Color {
    /// Black color
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Grey color
    pub const GREY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };

    /// Red color
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green color
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue color
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Yellow color
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Cyan color
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Magenta color
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// White color
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// creates a color from 8bit rgba (0-255)
    pub fn from_8bit_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// creates a color from 8bit rgb (0-255)
    pub fn from_8bit_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    /// creates a color from normalized floats (0.0-1.0)
    pub fn from_normalized(r: f32, g: f32, b: f32, a: f32) -> Color {
        // if !(0.0..=1.0).contains(&r)
        //     || !(0.0..=1.0).contains(&g)
        //     || !(0.0..=1.0).contains(&b)
        //     || !(0.0..=1.0).contains(&a)
        // {
        //     return Err("Normalized values must be between 0.0 and 1.0".to_string());
        // }
        Color { r, g, b, a }
    }

    /// creates a color from a hex value
    ///
    /// # Note
    /// if the color has a no alpha value its assumed that its rgb hex and alpha will be 1.0
    ///
    /// # Example
    /// ```rust
    /// use maple::color::Color;
    ///
    /// assert_eq!(Color::from_hex(0xFFFFFF), Color::from_normalized(1.0, 1.0, 1.0, 1.0))
    /// ```
    pub fn from_hex(hex: u32) -> Color {
        if hex <= 0xFFFFFF {
            // 24-bit RGB, default alpha to 255
            let r = ((hex >> 16) & 0xFF) as u8;
            let g = ((hex >> 8) & 0xFF) as u8;
            let b = (hex & 0xFF) as u8;
            Color::from_8bit_rgba(r, g, b, 255)
        } else {
            // 32-bit RGBA
            let r = ((hex >> 24) & 0xFF) as u8;
            let g = ((hex >> 16) & 0xFF) as u8;
            let b = ((hex >> 8) & 0xFF) as u8;
            let a = (hex & 0xFF) as u8;
            Color::from_8bit_rgba(r, g, b, a)
        }
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let a = math::Vec4::new(self.r, self.g, self.b, self.a);
        let b = math::Vec4::new(other.r, other.g, other.b, other.a);
        let c = a.lerp(b, t);
        Color {
            r: c.x,
            g: c.y,
            b: c.z,
            a: c.w,
        }
    }

    pub fn hdr(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub fn with_intensity(self, factor: f32) -> Color {
        Color {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }

    #[inline]
    pub fn with_alpha(self, alpha: f32) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    /// Converts from sRGB to linear space
    ///
    /// Use when loading colors from 8-bit sRGB sources (most image files, CSS hex values).
    /// Your PBR shader expects linear colors.
    pub fn to_linear(self) -> Color {
        let srgb_to_linear = |x: f32| -> f32 {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        Color {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Converts from linear to sRGB space
    ///
    /// Use before writing to an sRGB framebuffer if not using hardware sRGB conversion.
    pub fn to_srgb(self) -> Color {
        let linear_to_srgb = |x: f32| -> f32 {
            if x <= 0.0031308 {
                x * 12.92
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            }
        };
        Color {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }

    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

impl From<Color> for math::Vec4 {
    fn from(color: Color) -> Self {
        math::vec4(color.r, color.g, color.b, color.a)
    }
}

impl Into<Color> for Vec3 {
    fn into(self) -> Color {
        Color {
            r: self.x,
            g: self.y,
            b: self.z,
            a: 1.0,
        }
    }
}

impl From<math::Vec4> for Color {
    fn from(vec: math::Vec4) -> Self {
        Color {
            r: vec.x,
            g: vec.y,
            b: vec.z,
            a: vec.w,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(value: Color) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn test_from_8bit_rgba_valid() {
        let color = Color::from_8bit_rgba(255, 128, 64, 32);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 32.0 / 255.0);
    }

    #[test]
    fn test_from_normalized_valid() {
        let color = Color::from_normalized(0.0, 0.5, 1.0, 0.25);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 1.0);
        assert_eq!(color.a, 0.25);
    }

    #[test]
    fn test_from_hex_valid_with_alpha() {
        let color = Color::from_hex(0xFF8040A0); // Includes alpha channel

        println!("{:?}", color);

        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 160.0 / 255.0);
    }

    #[test]
    fn test_from_hex_valid_no_alpha() {
        let color = Color::from_hex(0xFF8040); // No alpha channel, defaults to 255

        println!("{:?}", color);

        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 1.0); // Default alpha is 255 (1.0 normalized)
    }

    #[test]
    fn test_conversion_to_vec4() {
        let color = Color {
            r: 0.5,
            g: 0.25,
            b: 0.75,
            a: 1.0,
        };
        let vec4: Vec4 = color.into();
        assert_eq!(vec4.x, 0.5);
        assert_eq!(vec4.y, 0.25);
        assert_eq!(vec4.z, 0.75);
        assert_eq!(vec4.w, 1.0);
    }

    #[test]
    fn test_conversion_from_vec4() {
        let vec4 = Vec4::new(0.5, 0.25, 0.75, 1.0);
        let color: Color = vec4.into();
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.25);
        assert_eq!(color.b, 0.75);
        assert_eq!(color.a, 1.0);
    }
}
