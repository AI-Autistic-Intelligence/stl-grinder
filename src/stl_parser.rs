use anyhow::{Context, Result};
use stl_io::{read_stl, IndexedMesh, Vector};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl BoundingBox {
    pub fn height(&self) -> f32 {
        self.max_z - self.min_z
    }
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }
    pub fn depth(&self) -> f32 {
        self.max_y - self.min_y
    }
}

pub struct StlLoader;

impl StlLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(IndexedMesh, BoundingBox)> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open STL file: {}", path.display()))?;

        let mesh = read_stl(&mut file)
            .with_context(|| "Failed to parse STL geometry")?;

        if mesh.vertices.is_empty() {
            anyhow::bail!("STL mesh contains zero vertices");
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for v in &mesh.vertices {
            min_x = min_x.min(v[0]);
            max_x = max_x.max(v[0]);
            min_y = min_y.min(v[1]);
            max_y = max_y.max(v[1]);
            min_z = min_z.min(v[2]);
            max_z = max_z.max(v[2]);
        }

        let bbox = BoundingBox {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        };

        Ok((mesh, bbox))
    }
}
