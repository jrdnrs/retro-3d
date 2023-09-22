macro_rules! count_items {
    ($name:ident) => { 1 };
    ($first:ident, $($rest:ident),*) => {
        1 + count_items!($($rest),*)
    }
}

/// We can't use '+' as the separator, so we include it at the start of the
/// repetition and then strip the initial '+'.
macro_rules! strip_plus {
    {+ $($rest:tt)* } => { $($rest)* }
}

macro_rules! def_vec {
    ( $name: ident { $($field: ident),+ } ) => {

        #[repr(C)]
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub struct $name {
            $(pub $field: f32),+
        }

    }
}

macro_rules! impl_core_vec {
    ( $name: ident { $($field: ident),+ } ) => {

        impl $name {
            pub const ZERO: Self = Self {
                $($field: 0.0),+
            };

            pub const ONE: Self = Self {
                $($field: 1.0),+
            };


            pub fn new( $($field: f32),+ ) -> Self {
                Self {
                    $($field),+
                }

            }

            pub fn uniform(a: f32) -> Self {
                Self {
                    $($field: a),+
                }
            }

            pub fn normalise(&self) -> Self {
                let m = self.magnitude();

                if m == 0.0 {
                    return Self::ZERO;
                }

                Self {
                    $($field: self.$field / m),+
                }
            }

            pub fn magnitude(&self) -> f32 {
                (
                    strip_plus!(
                        $(+ self.$field * self.$field)+
                    )
                ).sqrt()
            }

            pub fn magnitude_sq(&self) -> f32 {
                (
                    strip_plus!(
                        $(+ self.$field * self.$field)+
                    )
                )
            }

            pub fn dot(&self, rhs: Self) -> f32 {
                strip_plus!(
                    $(+ self.$field * rhs.$field)+
                )
            }

            pub fn sqrt(&self) -> Self {
                Self {
                    $($field: self.$field.sqrt()),+
                }
            }

            pub fn abs(&self) -> Self {
                Self {
                    $($field: self.$field.abs()),+
                }
            }

            pub fn recip(&self) -> Self {
                Self {
                    $($field: self.$field.recip()),+
                }
            }

            pub fn lerp(&self, rhs: Self, t: f32) -> Self {
                Self {
                    $($field: self.$field + (rhs.$field - self.$field) * t),+
                }
            }

            pub fn as_array(&self) -> &[f32; count_items!( $($field),+ )] {
                unsafe {
                    core::mem::transmute::<_, &[f32; count_items!( $($field),+ )]>(self)
                }
            }

            pub fn as_mut_array(&mut self) -> &mut [f32; count_items!( $($field),+ )] {
                unsafe {
                    core::mem::transmute::<_, &mut [f32; count_items!( $($field),+ )]>(self)
                }
            }

        }

        impl std::ops::Index<usize> for $name {
            type Output = f32;

            fn index(&self, i: usize) -> &Self::Output {
                self.as_array().index(i)
            }
        }

        impl std::ops::IndexMut<usize> for $name {
            fn index_mut(&mut self, i: usize) -> &mut Self::Output {
                self.as_mut_array().index_mut(i)
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field + rhs.$field),+
                }
            }
        }

        impl std::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field);+
            }
        }

        impl std::ops::Add<f32> for $name {
            type Output = Self;

            fn add(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field + rhs),+
                }
            }
        }

        impl std::ops::AddAssign<f32> for $name {
            fn add_assign(&mut self, rhs: f32) {
                $(self.$field += rhs);+
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field - rhs.$field),+
                }
            }
        }

        impl std::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field -= rhs.$field);+
            }
        }

        impl std::ops::Sub<f32> for $name {
            type Output = Self;

            fn sub(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field - rhs),+
                }
            }
        }

        impl std::ops::SubAssign<f32> for $name {
            fn sub_assign(&mut self, rhs: f32) {
                $(self.$field -= rhs);+
            }
        }

        impl std::ops::Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field * rhs.$field),+
                }
            }
        }

        impl std::ops::MulAssign for $name {
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$field *= rhs.$field);+
            }
        }

        impl std::ops::Mul<f32> for $name {
            type Output = Self;

            fn mul(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field * rhs),+
                }
            }
        }

        impl std::ops::MulAssign<f32> for $name {
            fn mul_assign(&mut self, rhs: f32) {
                $(self.$field *= rhs);+
            }
        }

        impl std::ops::Div for $name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field / rhs.$field),+
                }
            }
        }

        impl std::ops::DivAssign for $name {
            fn div_assign(&mut self, rhs: Self) {
                $(self.$field /= rhs.$field);+
            }
        }

        impl std::ops::Div<f32> for $name {
            type Output = Self;

            fn div(self, rhs: f32) -> Self::Output {
                Self {
                    $($field: self.$field / rhs),+
                }
            }
        }

        impl std::ops::DivAssign<f32> for $name {
            fn div_assign(&mut self, rhs: f32) {
                $(self.$field /= rhs);+
            }
        }


        impl std::ops::Neg for $name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    $($field: -self.$field),+
                }
            }
        }


    };
}

def_vec! { Vec2f {x, y} }
impl_core_vec! { Vec2f {x, y} }

impl Vec2f {
    pub fn perpendicular(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn cross(&self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn rotate(&self, sin: f32, cos: f32) -> Self {
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}

def_vec! { Vec3f {x, y, z} }
impl_core_vec! { Vec3f {x, y, z} }

impl Vec3f {
    pub fn cross(&self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl From<Vec2f> for Vec3f {
    fn from(v: Vec2f) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: 0.0,
        }
    }
}

impl From<Vec3f> for Vec2f {
    fn from(v: Vec3f) -> Self {
        Self { x: v.x, y: v.y }
    }
}
