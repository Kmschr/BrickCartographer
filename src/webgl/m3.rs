
pub fn translate(m: [f32; 9], tx: f32, ty: f32) -> [f32; 9] {
    multiply(m, translation(tx, ty))
}

pub fn rotate(m: [f32; 9], angle: f32) -> [f32; 9] {
    multiply(m, rotation(angle))
}

pub fn scale(m: [f32; 9], sx: f32, sy: f32) -> [f32; 9] {
    multiply(m, scaling(sx, sy))
}

pub fn translation(tx: f32, ty: f32) -> [f32; 9] {
    [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        tx,  ty,  1.0,
    ]
}

pub fn rotation(angle: f32) -> [f32; 9] {
    let c = angle.cos();
    let s = angle.sin();
    [
        c,  -s,   0.0,
        s,   c,   0.0,
        0.0, 0.0, 1.0,
    ]
}

pub fn scaling(sx: f32, sy: f32) -> [f32; 9] {
    [
        sx,  0.0, 0.0,
        0.0, sy,  0.0,
        0.0, 0.0, 1.0,
    ]
}

pub fn projection(width: f32, height: f32) -> [f32; 9] {
    [
        2.0 / width, 0.0, 0.0,
        0.0, -2.0 / height, 0.0,
        -1.0, 1.0, 1.0,
    ]
}

pub fn multiply(a: [f32; 9], b: [f32; 9]) -> [f32; 9] {
    let a00 = a[0];
    let a01 = a[1];
    let a02 = a[2];
    let a10 = a[3];
    let a11 = a[4];
    let a12 = a[5];
    let a20 = a[6];
    let a21 = a[7];
    let a22 = a[8];
    let b00 = b[0];
    let b01 = b[1];
    let b02 = b[2];
    let b10 = b[3];
    let b11 = b[4];
    let b12 = b[5];
    let b20 = b[6];
    let b21 = b[7];
    let b22 = b[8];
    [
        b00 * a00 + b01 * a10 + b02 * a20,
        b00 * a01 + b01 * a11 + b02 * a21,
        b00 * a02 + b01 * a12 + b02 * a22,
        b10 * a00 + b11 * a10 + b12 * a20,
        b10 * a01 + b11 * a11 + b12 * a21,
        b10 * a02 + b11 * a12 + b12 * a22,
        b20 * a00 + b21 * a10 + b22 * a20,
        b20 * a01 + b21 * a11 + b22 * a21,
        b20 * a02 + b21 * a12 + b22 * a22,
    ]
}

pub fn divide(m: [f32;9], x: f32) -> [f32;9] {
    [
        m[0]/x, m[1]/x, m[2]/x,
        m[3]/x, m[4]/x, m[5]/x,
        m[6]/x, m[7]/x, m[8]/x,
    ]
}

pub fn inverse(m: [f32;9]) -> [f32;9] {
    let det = determinant(m);
    divide(transpose(cofactor(m)), det)
}

pub fn determinant(m: [f32;9]) -> f32 {
    let a = m[0];
    let b = m[1];
    let c = m[2];
    let d = m[3];
    let e = m[4];
    let f = m[5];
    let g = m[6];
    let h = m[7];
    let i = m[8];
    a*(e*i - f*h) - b*(d*i - f*g) + c*(d*h - e*g)
}

pub fn transpose(m: [f32;9]) -> [f32;9] {
    [
        m[0], m[3], m[6],
        m[1], m[4], m[7],
        m[2], m[5], m[8],
    ]
}

pub fn cofactor(m: [f32;9]) -> [f32;9] {
    let m00 = m[0];
    let m01 = m[1];
    let m02 = m[2];
    let m10 = m[3];
    let m11 = m[4];
    let m12 = m[5];
    let m20 = m[6];
    let m21 = m[7];
    let m22 = m[8];
    let a00 = det2d([
        m11, m12,
        m21, m22,
    ]);
    let a01 = -det2d([
        m10, m12,
        m20, m22,
    ]);
    let a02 = det2d([
        m21, m11,
        m20, m21,
    ]);
    let a10 = -det2d([
        m01, m02,
        m21, m22,
    ]);
    let a11 = det2d([
        m00, m02,
        m20, m22,
    ]);
    let a12 = -det2d([
        m00, m01,
        m20, m21,
    ]);
    let a20 = det2d([
        m01, m02,
        m11, m12,
    ]);
    let a21 = -det2d([
        m00, m02,
        m10, m12,
    ]);
    let a22 = det2d([
        m00, m01,
        m10, m11,
    ]);
    [
        a00, a01, a02,
        a10, a11, a12,
        a20, a21, a22,
    ]
}

fn det2d(m: [f32;4]) -> f32 {
    m[0]*m[3] - m[1]*m[2]
}
