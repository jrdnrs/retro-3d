use super::vec::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
macro_rules! tranpose_mat {
    ($mat: ident, Vec2f) => {
        [
            $mat.array[0], $mat.array[2],
            $mat.array[1], $mat.array[3],
        ]
    };

    ($mat: ident, Vec3f) => {
        [
            $mat.array[0], $mat.array[3], $mat.array[6],
            $mat.array[1], $mat.array[4], $mat.array[7],
            $mat.array[2], $mat.array[5], $mat.array[8],
        ]
    };

    ($mat: ident, Vec4f) => {
        [
            $mat.array[0], $mat.array[4], $mat.array[8], $mat.array[12],
            $mat.array[1], $mat.array[5], $mat.array[9], $mat.array[13],
            $mat.array[2], $mat.array[6], $mat.array[10], $mat.array[14],
            $mat.array[3], $mat.array[7], $mat.array[11], $mat.array[15],
        ]
    };
}

macro_rules! def_mat {
    ( $name: ident, $vec: ident, $dim: expr ) => {
        #[repr(C)]
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub struct $name {
            array: [f32; $dim * $dim],
        }
    };
}

macro_rules! impl_core_mat {
    ( $name: ident, $vec: ident, $dim: expr ) => {
        impl $name {
            pub const ZERO: Self = Self {
                array: [0.0; $dim * $dim],
            };

            pub const IDENTITY: Self = {
                let mut mat = Self::ZERO;
                let mut i = 0;
                while i < $dim {
                    mat.array[i * $dim + i] = 1.0;
                    i += 1;
                }
                mat
            };

            pub fn new(a: [f32; $dim * $dim]) -> Self {
                Self { array: a }
            }

            pub fn from_columns(columns: [$vec; $dim]) -> Self {
                Self {
                    array: unsafe { core::mem::transmute(columns) },
                }
            }

            pub fn from_rows(rows: [$vec; $dim]) -> Self {
                Self::from_columns(rows).transpose()
            }

            pub fn uniform(a: f32) -> Self {
                Self {
                    array: [a; $dim * $dim],
                }
            }

            pub fn transpose(&self) -> Self {
                Self {
                    array: tranpose_mat!(self, $vec),
                }
            }

            pub fn sqrt(&self) -> Self {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i].sqrt();
                }

                matrix
            }

            pub fn abs(&self) -> Self {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i].abs();
                }

                matrix
            }

            pub fn recip(&self) -> Self {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = if self.array[i] == 0.0 {
                        0.0
                    } else {
                        1.0 / self.array[i]
                    };
                }

                matrix
            }

            pub fn as_array(&self) -> &[f32; $dim * $dim] {
                &self.array
            }

            pub fn as_mut_array(&mut self) -> &mut [f32; $dim * $dim] {
                &mut self.array
            }

            pub fn as_columns(&self) -> &[$vec; $dim] {
                unsafe { core::mem::transmute(&self.array) }
            }

            pub fn as_mut_columns(&mut self) -> &mut [$vec; $dim] {
                unsafe { core::mem::transmute(&mut self.array) }
            }
        }

        impl std::ops::Index<usize> for $name {
            type Output = $vec;

            fn index(&self, col: usize) -> &Self::Output {
                self.as_columns().index(col)
            }
        }

        impl std::ops::IndexMut<usize> for $name {
            fn index_mut(&mut self, col: usize) -> &mut Self::Output {
                self.as_mut_columns().index_mut(col)
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] + rhs.array[i];
                }

                matrix
            }
        }

        impl std::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                for i in 0..($dim * $dim) {
                    self.array[i] += rhs.array[i];
                }
            }
        }

        impl std::ops::Add<f32> for $name {
            type Output = Self;

            fn add(self, rhs: f32) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] + rhs;
                }

                matrix
            }
        }

        impl std::ops::AddAssign<f32> for $name {
            fn add_assign(&mut self, rhs: f32) {
                for i in 0..($dim * $dim) {
                    self.array[i] += rhs;
                }
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] - rhs.array[i];
                }

                matrix
            }
        }

        impl std::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                for i in 0..($dim * $dim) {
                    self.array[i] -= rhs.array[i];
                }
            }
        }

        impl std::ops::Sub<f32> for $name {
            type Output = Self;

            fn sub(self, rhs: f32) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] - rhs;
                }

                matrix
            }
        }

        impl std::ops::SubAssign<f32> for $name {
            fn sub_assign(&mut self, rhs: f32) {
                for i in 0..($dim * $dim) {
                    self.array[i] -= rhs;
                }
            }
        }

        impl std::ops::Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..$dim {
                    for j in 0..$dim {
                        for k in 0..$dim {
                            matrix[j][i] += self[k][i] * rhs[j][k];
                        }
                    }
                }

                matrix
            }
        }

        impl std::ops::MulAssign for $name {
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl std::ops::Mul<f32> for $name {
            type Output = Self;

            fn mul(self, rhs: f32) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] * rhs;
                }

                matrix
            }
        }

        impl std::ops::MulAssign<f32> for $name {
            fn mul_assign(&mut self, rhs: f32) {
                for i in 0..($dim * $dim) {
                    self.array[i] *= rhs;
                }
            }
        }

        impl std::ops::Mul<$vec> for $name {
            type Output = $vec;

            fn mul(self, rhs: $vec) -> Self::Output {
                let mut vector = $vec::ZERO;

                for i in 0..$dim {
                    for j in 0..$dim {
                        vector[i] += self[j][i] * rhs[j];
                    }
                }

                vector
            }
        }

        impl std::ops::Div<f32> for $name {
            type Output = Self;

            fn div(self, rhs: f32) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = self.array[i] / rhs;
                }

                matrix
            }
        }

        impl std::ops::DivAssign<f32> for $name {
            fn div_assign(&mut self, rhs: f32) {
                for i in 0..($dim * $dim) {
                    self.array[i] /= rhs;
                }
            }
        }

        impl std::ops::Neg for $name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                let mut matrix = Self::ZERO;

                for i in 0..($dim * $dim) {
                    matrix.array[i] = -self.array[i];
                }

                matrix
            }
        }
    };
}

def_mat! { Mat2f, Vec2f, 2 }
impl_core_mat! { Mat2f, Vec2f, 2 }

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Mat2f {
    pub fn rotation(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();

        Self {
            array: [
                cos,  sin,
                -sin,  cos,
            ]
        }
    }

    pub fn scale(scale: Vec2f) -> Self {
        Self {
            array: [
                scale.x, 0.0,
                0.0, scale.y,
            ]
        }
    }
    
    pub fn determinant(&self) -> f32 {
        self[0][0] * self[1][1] - self[1][0] * self[0][1]
    }

    pub fn inverse(&self) -> Self {
        let det = self.determinant();

        if det == 0.0 {
            panic!("Matrix is not invertible");
        }

        let mut matrix = Self::ZERO;

        matrix[0][0] = self[1][1];
        matrix[0][1] = -self[0][1];
        matrix[1][0] = -self[1][0];
        matrix[1][1] = self[0][0];

        let denom = 1.0 / det;
        matrix *= denom;

        return matrix;
    }
}

def_mat! { Mat3f, Vec3f, 3 }
impl_core_mat! { Mat3f, Vec3f, 3 }

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Mat3f {
    pub fn determinant(&self) -> f32 {
        let (a, b, c) = (self[0][0], self[0][1], self[0][2]);
        let (d, e, f) = (self[1][0], self[1][1], self[1][2]);
        let (g, h, i) = (self[2][0], self[2][1], self[2][2]);

          a * (e * i - f * h) 
        - b * (d * i - f * g) 
        + c * (d * h - e * g)
    }

    pub fn inverse(&self) -> Self {
        let (a, b, c) = (self[0][0], self[0][1], self[0][2]);
        let (d, e, f) = (self[1][0], self[1][1], self[1][2]);
        let (g, h, i) = (self[2][0], self[2][1], self[2][2]);

        let det =   a * (e * i - f * h) 
                       - b * (d * i - f * g) 
                       + c * (d * h - e * g);

        if det == 0.0 {
            panic!("Matrix is not invertible");
        }

        let mut matrix = Self::ZERO;

        matrix[0][0] =   e * i - f * h;
        matrix[0][1] = -(b * i - c * h);
        matrix[0][2] =   b * f - c * e;
        matrix[1][0] = -(d * i - f * g);
        matrix[1][1] =   a * i - c * g;
        matrix[1][2] = -(a * f - c * d);
        matrix[2][0] =   d * h - e * g;
        matrix[2][1] = -(a * h - b * g);
        matrix[2][2] =   a * e - b * d;

        let denom = 1.0 / det;
        matrix *= denom;

        return matrix;
    }
}
