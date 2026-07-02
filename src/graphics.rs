// Interleaves 2D positions with a color into the packed vertex layout:
// x: f32, y: f32, rgba: 4 x u8 (12 bytes per vertex, matches VERTEX_STRIDE)
pub fn push_vertices(buffer: &mut Vec<u8>, positions: &[f32], color: [u8; 4]) {
    for pos in positions.chunks_exact(2) {
        buffer.extend_from_slice(&pos[0].to_le_bytes());
        buffer.extend_from_slice(&pos[1].to_le_bytes());
        buffer.extend_from_slice(&color);
    }
}

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
