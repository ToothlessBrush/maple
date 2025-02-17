use nalgebra_glm as glm;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    // Constructor from 8-bit RGBA (0-255) with validation
    pub fn from_8bit_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn from_8bit_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

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
}

impl From<Color> for glm::Vec4 {
    fn from(color: Color) -> Self {
        glm::vec4(color.r, color.g, color.b, color.a)
    }
}

impl From<glm::Vec4> for Color {
    fn from(vec: glm::Vec4) -> Self {
        Color {
            r: vec.x,
            g: vec.y,
            b: vec.z,
            a: vec.w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::Vec4;

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
