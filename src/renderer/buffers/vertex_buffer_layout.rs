//! the layout stores the layout of the vertex buffer for example a vertex may have 3 floats for position and 2 floats for texture coordinates the layout stores this information

/// stores the element of the vertex buffer
pub struct VertexBufferElement {
    /// the count of the element
    pub count: i32,
    /// the gl type of the element
    pub type_: u32,
    /// if the element is normalized
    pub normalized: bool,
}

impl VertexBufferElement {
    /// Gets the size of the type
    ///
    /// # Arguments
    /// - `type_` - the type to get the size of
    pub fn size_of_type(type_: u32) -> i32 {
        match type_ {
            gl::FLOAT => std::mem::size_of::<f32>() as i32,
            gl::UNSIGNED_INT => std::mem::size_of::<u32>() as i32,
            gl::UNSIGNED_BYTE => std::mem::size_of::<u8>() as i32,
            _ => 0,
        }
    }
}

/// stores the layout of the vertex buffer
pub struct VertexBufferLayout {
    /// the elements of the layout
    pub elements: Vec<VertexBufferElement>,
    /// the stride of the layout (the size of the vertex)
    pub stride: i32,
}

impl Default for VertexBufferLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexBufferLayout {
    /// Creates a new vertex buffer layout
    pub fn new() -> VertexBufferLayout {
        VertexBufferLayout {
            elements: Vec::new(),
            stride: 0,
        }
    }

    /// Pushes a new element to the layout with the specified count.
    ///
    /// # Arguments
    /// - `count` - the count of the element
    pub fn push<T: VertexAttrib>(&mut self, count: i32) {
        self.elements.push(VertexBufferElement {
            type_: T::get_type(),
            count,
            normalized: T::is_normalized(),
        });
        self.stride += VertexBufferElement::size_of_type(T::get_type()) * count;
    }

    /// Pushes a new mat4 to the layout.
    pub fn push_mat4(&mut self) {
        for _ in 0..4 {
            // 4x4 floats in a mat4
            self.push::<f32>(4);
        }
    }
}

/// The vertex attribute trait
pub trait VertexAttrib {
    /// Gets the type of the vertex attribute (for conversion from rust types to gl types)
    ///
    /// # Returns
    /// the type of the vertex attribute
    fn get_type() -> u32;

    /// Checks if the vertex attribute is normalized
    ///
    /// # Returns
    /// if the vertex attribute is normalized
    fn is_normalized() -> bool;
}

impl VertexAttrib for f32 {
    fn get_type() -> u32 {
        gl::FLOAT
    }

    fn is_normalized() -> bool {
        false
    }
}

impl VertexAttrib for u32 {
    fn get_type() -> u32 {
        gl::UNSIGNED_INT
    }

    fn is_normalized() -> bool {
        false
    }
}

impl VertexAttrib for u8 {
    fn get_type() -> u32 {
        gl::UNSIGNED_BYTE
    }

    fn is_normalized() -> bool {
        true
    }
}
