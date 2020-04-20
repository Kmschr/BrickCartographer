use crate::Color;

#[derive(Debug)]
pub struct RenderObject {
    pub vertices: Vec<f32>,
    pub color: Color
}

impl RenderObject {
    pub fn get_vertex_array(&self) -> Vec<f32> {
        let mut vertex_array = Vec::new();
        let vertex_count = self.vertices.len() / 2;

        for i in 0..vertex_count {
            vertex_array.push(self.vertices[i*2]);
            vertex_array.push(self.vertices[i*2 + 1]);
            vertex_array.push(self.color.r);
            vertex_array.push(self.color.g);
            vertex_array.push(self.color.b);
        }

        vertex_array
    }
}

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
