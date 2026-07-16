use brickadia::save::{Direction, Rotation};

/// A render-ready brick, converted from whichever save format at load time:
/// filtered to visible bricks, size transformed for rotation/direction, and
/// color resolved to display sRGB.
///
/// Kept deliberately small (28 bytes vs brickadia's 112-byte `Brick`): saves
/// run to tens of millions of bricks and the whole list must fit alongside
/// everything else under wasm32's 4GB memory ceiling.
pub struct Brick {
    pub position: (i32, i32, i32),
    pub size: (u16, u16, u16),
    pub asset_name_index: u32,
    /// Display color as sRGB rgba bytes
    pub color: [u8; 4],
    pub rotation: Rotation,
    pub direction: Direction,
}

impl Brick {
    pub fn size_u32(&self) -> (u32, u32, u32) {
        (self.size.0 as u32, self.size.1 as u32, self.size.2 as u32)
    }
}
