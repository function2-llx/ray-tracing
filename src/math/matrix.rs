use std::ops;

use crate::math::vector::Vector3f;
use crate::math::FloatT;
use serde::Deserialize;

// Matrix with order 3
#[repr(C)]
#[derive(Copy, Clone, Deserialize)]
pub struct Matrix3([[FloatT; 3]; 3]);

impl std::ops::Deref for Matrix3 {
    type Target = [[FloatT; 3]; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Matrix3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Matrix3 {
    // column: 是否为列主序
    pub fn from_vectors(vectors: [Vector3f; 3], column: bool) -> Self {
        // fill with row major order
        let mut ret: Matrix3 = unsafe { std::mem::transmute(vectors) };
        if column {
            ret = ret.transposed();
        }
        ret
    }
    pub fn row(&self, i: usize) -> Vector3f {
        Vector3f::new([self[i][0], self[i][1], self[i][2]])
    }
    pub fn set_row(&mut self, i: usize, x: &Vector3f) {
        for j in 0..3 {
            self[i][j] = x[j];
        }
    }

    pub fn set_column(&mut self, j: usize, x: &Vector3f) {
        for i in 0..3 {
            self[i][j] = x[i];
        }
    }

    pub fn column(&self, j: usize) -> Vector3f {
        Vector3f::new([self[0][j], self[1][j], self[2][j]])
    }

    pub fn transposed(&self) -> Matrix3 {
        Matrix3([
            [self[0][0], self[1][0], self[2][0]],
            [self[0][1], self[1][1], self[2][1]],
            [self[0][2], self[1][2], self[2][2]],
        ])
    }
}

impl ops::Mul<Vector3f> for Matrix3 {
    type Output = Vector3f;

    fn mul(self, rhs: Vector3f) -> Self::Output {
        Vector3f::new([
            Vector3f::dot(&self.row(0), &rhs),
            Vector3f::dot(&self.row(1), &rhs),
            Vector3f::dot(&self.row(2), &rhs),
        ])
    }
}
