pub struct VertexBufferElement {
    pub count: i32,
    pub type_: u32,
    pub normalized: bool,
}

impl VertexBufferElement {
    pub fn size_of_type(type_: u32) -> i32 {
        match type_ {
            gl::FLOAT => std::mem::size_of::<f32>() as i32,
            gl::UNSIGNED_INT => std::mem::size_of::<u32>() as i32,
            gl::UNSIGNED_BYTE => std::mem::size_of::<u8>() as i32,
            _ => 0,
        }
    }
}

pub struct VertexBufferLayout {
    pub elements: Vec<VertexBufferElement>,
    pub stride: i32,
}

impl VertexBufferLayout {
    pub fn new() -> VertexBufferLayout {
        VertexBufferLayout {
            elements: Vec::new(),
            stride: 0,
        }
    }

    pub fn push<T: VertexAttrib>(&mut self, count: i32) {
        self.elements.push(VertexBufferElement {
            type_: T::get_type(),
            count,
            normalized: T::is_normalized(),
        });
        self.stride += VertexBufferElement::size_of_type(T::get_type()) * count;
    }
}

pub trait VertexAttrib {
    fn get_type() -> u32;
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