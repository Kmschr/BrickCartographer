use std::collections::HashMap;

// Bytes per vertex: x (f32), y (f32), rgba (4 x u8)
pub const VERTEX_STRIDE: i32 = 12;

// Appends a shape's triangle-list positions as indexed geometry, deduplicating
// vertices within the shape by exact bit pattern. Vertices are interleaved as
// x: f32, y: f32, rgba: 4 x u8 (12 bytes per vertex, matches VERTEX_STRIDE).
pub fn push_shape(vertices: &mut Vec<u8>, indices: &mut Vec<u32>, positions: &[f32], color: [u8; 4]) {
    let mut index_of: HashMap<(u32, u32), u32> = HashMap::new();
    for pos in positions.chunks_exact(2) {
        let key = (pos[0].to_bits(), pos[1].to_bits());
        let index = *index_of.entry(key).or_insert_with(|| {
            let index = (vertices.len() / VERTEX_STRIDE as usize) as u32;
            vertices.extend_from_slice(&pos[0].to_le_bytes());
            vertices.extend_from_slice(&pos[1].to_le_bytes());
            vertices.extend_from_slice(&color);
            index
        });
        indices.push(index);
    }
}
