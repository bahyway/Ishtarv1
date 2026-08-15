//! Software renderer — produces PNG frames from projected particle data.
//! This path requires no GPU passthrough and works inside the Hyper-V VM.
//! The GPU (Ash/wgpu) path is enabled with the `vulkan-backend` feature.

use anyhow::Result;
use image::{ImageBuffer, Rgba};

use crate::particle::Point3D;

pub struct SoftwareRenderer {
    width: u32,
    height: u32,
}

impl SoftwareRenderer {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self { width, height })
    }

    /// Render a slice of projected points to an RGBA image.
    pub fn render_frame(
        &self,
        points: &[Point3D],
        step: u64,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let mut img = ImageBuffer::from_pixel(
            self.width,
            self.height,
            Rgba([12u8, 12, 18, 255]), // dark background
        );

        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let scale = (self.width.min(self.height)) as f32 / 2.5;

        for p in points {
            // Simple orthographic projection: ignore z, map x/y to pixel
            let px = (cx + p.x * scale) as i32;
            let py = (cy - p.y * scale) as i32;

            if px < 0 || px >= self.width as i32 || py < 0 || py >= self.height as i32 {
                continue;
            }

            let pixel = Rgba([
                (p.r * 255.0) as u8,
                (p.g * 255.0) as u8,
                (p.b * 255.0) as u8,
                (p.a * 255.0) as u8,
            ]);

            // Splat a 2×2 quad for visibility at 1M particles
            for dy in 0..2i32 {
                for dx in 0..2i32 {
                    let nx = px + dx;
                    let ny = py + dy;
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        img.put_pixel(nx as u32, ny as u32, pixel);
                    }
                }
            }
        }

        // Overlay step counter (simple text, no font dep)
        self.draw_step_number(&mut img, step);

        Ok(img)
    }

    fn draw_step_number(&self, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, step: u64) {
        // White 3×5 pixel dot as a progress marker (no font needed)
        let x = 10 + (step as u32 % 200) * 3;
        let y = 10u32;
        for dy in 0..5u32 {
            for dx in 0..3u32 {
                if x + dx < self.width && y + dy < self.height {
                    img.put_pixel(x + dx, y + dy, Rgba([255, 255, 255, 200]));
                }
            }
        }
    }

    /// Encode an image as a base64 PNG string (for Jupyter display_data)
    pub fn to_base64_png(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String> {
        use base64::Engine;
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particle::ParticleCloud;
    use crate::projection::Projection7D3D;

    #[test]
    fn renders_without_panic() {
        let cloud = ParticleCloud::new_storage_fragmentation(1000);
        let proj = Projection7D3D::default();
        let points = cloud.project(&proj);

        let renderer = SoftwareRenderer::new(200, 150).unwrap();
        let frame = renderer.render_frame(&points, 0).unwrap();
        assert_eq!(frame.width(), 200);
        assert_eq!(frame.height(), 150);
    }
}
