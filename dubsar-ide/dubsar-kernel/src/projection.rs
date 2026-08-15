use serde::{Deserialize, Serialize};

use crate::particle::Particle7D;

/// Projects a 7D particle to 3D for rendering.
///
/// The 7D state vector is: [pos.x, pos.y, pos.z, mom.x, mom.y, mom.z, scalar]
/// The projection is a 3×7 matrix: each output dimension is a linear combination
/// of the 7D state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Projection7D3D {
    /// 3 basis vectors in 7D (rows = output axes, cols = input dims)
    pub basis: [[f32; 7]; 3],
    /// Translation in 7D before projection
    pub origin: [f32; 7],
    /// Global scale applied after projection
    pub scale: f32,
}

impl Default for Projection7D3D {
    fn default() -> Self {
        // Default: use the 3 spatial dimensions directly
        Self {
            basis: [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            ],
            origin: [0.0; 7],
            scale: 0.01, // Scale so 200-unit address space → ±2 NDC units
        }
    }
}

impl Projection7D3D {
    /// Projection that visualises momentum magnitude on the Z axis
    pub fn position_momentum_z() -> Self {
        Self {
            basis: [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.5, 0.0, 0.0, 0.5, 0.0], // mix pos.z + mom.z
            ],
            origin: [0.0; 7],
            scale: 0.01,
        }
    }

    /// Phase-space projection: pos.x vs mom.x vs fragmentation scalar
    pub fn phase_space() -> Self {
        Self {
            basis: [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // X = position.x
                [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], // Y = momentum.x
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // Z = scalar
            ],
            origin: [0.0; 7],
            scale: 0.01,
        }
    }

    /// Project a single 7D particle to 3D
    pub fn project(&self, p: &Particle7D) -> [f32; 3] {
        let v: [f32; 7] = [
            p.position[0] - self.origin[0],
            p.position[1] - self.origin[1],
            p.position[2] - self.origin[2],
            p.momentum[0] - self.origin[3],
            p.momentum[1] - self.origin[4],
            p.momentum[2] - self.origin[5],
            p.scalar - self.origin[6],
        ];

        let dot = |row: &[f32; 7]| -> f32 {
            row.iter().zip(v.iter()).map(|(a, b)| a * b).sum()
        };

        [
            dot(&self.basis[0]) * self.scale,
            dot(&self.basis[1]) * self.scale,
            dot(&self.basis[2]) * self.scale,
        ]
    }

    /// 4×4 column-major matrix representation for GPU upload
    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        [
            [self.basis[0][0], self.basis[1][0], self.basis[2][0], 0.0],
            [self.basis[0][1], self.basis[1][1], self.basis[2][1], 0.0],
            [self.basis[0][2], self.basis[1][2], self.basis[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
