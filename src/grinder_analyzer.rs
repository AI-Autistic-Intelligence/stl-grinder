use crate::stl_parser::BoundingBox;
use stl_io::IndexedMesh;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrinderChamberConfig {
    pub z_split_mm: f32,
    pub chamber_radius_mm: f32,
    pub max_allowed_radius_mm: f32,
    pub chamber_height_mm: f32,
    pub wall_thickness_mm: f32,
    pub magnet_enabled: bool,
    pub magnet_diameter_mm: f32,
    pub magnet_depth_mm: f32,
    pub teeth_rings_count: usize,
    pub recess_teeth: bool,
    pub teeth_recess_clearance_mm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnalysis {
    pub min_z: f32,
    pub max_z: f32,
    pub width_mm: f32,
    pub depth_mm: f32,
    pub height_mm: f32,
    pub optimal_z_split_mm: f32,
    pub max_safe_chamber_radius_mm: f32,
    pub suggested_config: GrinderChamberConfig,
}

pub struct MeshAnalyzer;

impl MeshAnalyzer {
    /// Analyzes an upright 3D STL model to find the widest circular horizontal cross-section.
    pub fn analyze(mesh: &IndexedMesh, bbox: &BoundingBox, wall_thickness_mm: f32) -> MeshAnalysis {
        let center_x = (bbox.min_x + bbox.max_x) / 2.0;
        let center_y = (bbox.min_y + bbox.max_y) / 2.0;

        let num_slices = 50;
        let z_step = bbox.height() / (num_slices as f32);

        let mut best_z = bbox.min_z + (bbox.height() * 0.5);
        let mut max_inscribed_r = 0.0f32;

        // Scan Z slices to find the largest horizontal inscribed circle
        for i in 1..num_slices {
            let z_plane = bbox.min_z + (i as f32 * z_step);

            // Find minimum distance from center (cx, cy) to any vertex within a Z-band around z_plane
            let mut min_r_at_z = f32::MAX;
            let mut count = 0;

            for v in &mesh.vertices {
                if (v[2] - z_plane).abs() <= z_step * 1.5 {
                    let dx = v[0] - center_x;
                    let dy = v[1] - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < min_r_at_z {
                        min_r_at_z = dist;
                    }
                    count += 1;
                }
            }

            if count > 5 && min_r_at_z != f32::MAX && min_r_at_z > max_inscribed_r {
                max_inscribed_r = min_r_at_z;
                best_z = z_plane;
            }
        }

        // Fallback if mesh has uniform bounding box
        if max_inscribed_r <= 0.0 {
            max_inscribed_r = (bbox.width().min(bbox.depth()) / 2.0) * 0.9;
        }

        let max_safe_r = (max_inscribed_r - wall_thickness_mm).max(5.0);
        let default_r = max_safe_r * 0.9;

        let suggested_config = GrinderChamberConfig {
            z_split_mm: best_z,
            chamber_radius_mm: default_r,
            max_allowed_radius_mm: max_safe_r,
            chamber_height_mm: 10.0, // Default 1 cm height
            wall_thickness_mm,
            magnet_enabled: true,
            magnet_diameter_mm: 3.1, // 3mm + 0.1mm tolerance
            magnet_depth_mm: 2.1,    // 2mm + 0.1mm tolerance
            teeth_rings_count: 3,
            recess_teeth: true,
            teeth_recess_clearance_mm: 0.5,
        };

        MeshAnalysis {
            min_z: bbox.min_z,
            max_z: bbox.max_z,
            width_mm: bbox.width(),
            depth_mm: bbox.depth(),
            height_mm: bbox.height(),
            optimal_z_split_mm: best_z,
            max_safe_chamber_radius_mm: max_safe_r,
            suggested_config,
        }
    }
}
