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
    pub fn from_8bit_rgba(r: u32, g: u32, b: u32, a: u32) -> Result<Color, String> {
        if r > 255 || g > 255 || b > 255 || a > 255 {
            return Err("RGB or Alpha value out of range (0-255)".to_string());
        }
        Ok(Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        })
    }

    pub fn from_8bit_rgb(r: u32, g: u32, b: u32) -> Result<Color, String> {
        if r > 255 || g > 255 || b > 255 {
            return Err("RGB value out of range (0-255)".to_string());
        }

        Ok(Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        })
    }

    pub fn from_normalized(r: f32, g: f32, b: f32, a: f32) -> Result<Color, String> {
        if !(0.0..=1.0).contains(&r)
            || !(0.0..=1.0).contains(&g)
            || !(0.0..=1.0).contains(&b)
            || !(0.0..=1.0).contains(&a)
        {
            return Err("Normalized values must be between 0.0 and 1.0".to_string());
        }
        Ok(Color { r, g, b, a })
    }

    pub fn from_hex(hex: &str) -> Result<Color, String> {
        if hex.len() != 9 && hex.len() != 7 {
            return Err("Invalid hex format".to_string());
        }

        let r = u8::from_str_radix(&hex[1..3], 16).map_err(|_| "Invalid hex value")?;
        let g = u8::from_str_radix(&hex[3..5], 16).map_err(|_| "Invalid hex value")?;
        let b = u8::from_str_radix(&hex[5..7], 16).map_err(|_| "Invalid hex value")?;
        let a = if hex.len() == 9 {
            u8::from_str_radix(&hex[7..9], 16).map_err(|_| "Invalid hex value")?
        } else {
            255
        };

        Color::from_8bit_rgba(r as u32, g as u32, b as u32, a as u32)
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
    use nalgebra_glm::Vec4; // Import everything from the parent module

    #[test]
    fn test_from_8bit_rgba_valid() {
        let color = Color::from_8bit_rgba(255, 128, 64, 32).unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 32.0 / 255.0);
    }

    #[test]
    fn test_from_8bit_rgba_invalid() {
        let result = Color::from_8bit_rgba(300, 0, 0, 255);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "RGB or Alpha value out of range (0-255)"
        );
    }

    #[test]
    fn test_from_normalized_valid() {
        let color = Color::from_normalized(0.0, 0.5, 1.0, 0.25).unwrap();
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 1.0);
        assert_eq!(color.a, 0.25);
    }

    #[test]
    fn test_from_normalized_invalid() {
        let result = Color::from_normalized(-0.1, 0.0, 1.2, 0.5);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Normalized values must be between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_from_hex_valid() {
        let color = Color::from_hex("#FF8040A0").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 160.0 / 255.0);
    }

    #[test]
    fn test_from_hex_valid_no_alpha() {
        let color = Color::from_hex("#FF8040").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 128.0 / 255.0);
        assert_eq!(color.b, 64.0 / 255.0);
        assert_eq!(color.a, 1.0); // Default alpha is 255 (1.0 normalized)
    }

    #[test]
    fn test_from_hex_invalid_format() {
        let result = Color::from_hex("FF8040A0"); // Missing '#'
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid hex format");
    }

    #[test]
    fn test_from_hex_invalid_characters() {
        let result = Color::from_hex("#FF80ZZ40");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid hex value");
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
