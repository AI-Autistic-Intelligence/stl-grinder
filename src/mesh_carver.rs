use crate::grinder_analyzer::GrinderChamberConfig;
use crate::stl_parser::BoundingBox;
use crate::teeth_generator::TeethGenerator;
use stl_io::{IndexedMesh, IndexedTriangle, Vector};
use std::f32::consts::PI;

pub struct CarvedGrinderModel {
    pub top_mesh: IndexedMesh,
    pub bottom_mesh: IndexedMesh,
    pub combined_mesh: IndexedMesh,
    pub top_triangle_count: usize,
    pub bottom_triangle_count: usize,
}

pub struct MeshCarver;

impl MeshCarver {
    /// Carves the 3D STL model into 2 interlocking Top and Bottom pieces with Gizeh teeth and magnet pockets.
    pub fn carve_grinder_model(
        mesh: &IndexedMesh,
        bbox: &BoundingBox,
        config: &GrinderChamberConfig,
    ) -> CarvedGrinderModel {
        let center_x = (bbox.min_x + bbox.max_x) / 2.0;
        let center_y = (bbox.min_y + bbox.max_y) / 2.0;
        let z_split = config.z_split_mm;

        let half_height = config.chamber_height_mm / 2.0;
        let top_chamber_floor_z = z_split + half_height;
        let bottom_chamber_floor_z = z_split - half_height;

        // 1. Build Top Piece
        let mut top_mesh = Self::carve_piece(
            mesh,
            center_x,
            center_y,
            z_split,
            top_chamber_floor_z,
            config.chamber_radius_mm,
            config.chamber_height_mm,
            true, // Top Piece
            config,
        );

        // 2. Build Bottom Piece
        let mut bottom_mesh = Self::carve_piece(
            mesh,
            center_x,
            center_y,
            z_split,
            bottom_chamber_floor_z,
            config.chamber_radius_mm,
            config.chamber_height_mm,
            false, // Bottom Piece
            config,
        );

        let top_count = top_mesh.faces.len();
        let bottom_count = bottom_mesh.faces.len();

        // 3. Build Combined Non-Overlapping Side-by-Side Mesh (Centered around origin at Z=0)
        let max_bbox_radius = ((bbox.max_x - bbox.min_x).max(bbox.max_y - bbox.min_y)) / 2.0;
        let offset_x = (config.chamber_radius_mm.max(max_bbox_radius)) + 15.0;
        let mut combined_vertices = Vec::new();
        let mut combined_faces = Vec::new();

        // Add Top Mesh (shifted to -offset_x, split face at Z=0)
        let top_v_offset = combined_vertices.len();
        for v in &top_mesh.vertices {
            combined_vertices.push(Vector::new([
                v[0] - center_x - offset_x,
                v[1] - center_y,
                v[2] - z_split,
            ]));
        }
        for f in &top_mesh.faces {
            combined_faces.push(IndexedTriangle {
                normal: f.normal,
                vertices: [
                    f.vertices[0] + top_v_offset,
                    f.vertices[1] + top_v_offset,
                    f.vertices[2] + top_v_offset,
                ],
            });
        }

        // Add Bottom Mesh (shifted to +offset_x, flipped so split face sits flat at Z=0)
        let bottom_v_offset = combined_vertices.len();
        for v in &bottom_mesh.vertices {
            combined_vertices.push(Vector::new([
                v[0] - center_x + offset_x,
                v[1] - center_y,
                z_split - v[2],
            ]));
        }
        for f in &bottom_mesh.faces {
            combined_faces.push(IndexedTriangle {
                normal: Vector::new([f.normal[0], f.normal[1], -f.normal[2]]),
                vertices: [
                    f.vertices[0] + bottom_v_offset,
                    f.vertices[2] + bottom_v_offset, // Inverted winding for flipped normal
                    f.vertices[1] + bottom_v_offset,
                ],
            });
        }

        let combined_mesh = IndexedMesh {
            vertices: combined_vertices,
            faces: combined_faces,
        };

        // Center and ground individual top and bottom meshes around origin (0,0) at Z=0
        for v in &mut top_mesh.vertices {
            *v = Vector::new([v[0] - center_x, v[1] - center_y, v[2] - z_split]);
        }

        for v in &mut bottom_mesh.vertices {
            *v = Vector::new([v[0] - center_x, v[1] - center_y, z_split - v[2]]);
        }

        CarvedGrinderModel {
            top_mesh,
            bottom_mesh,
            combined_mesh,
            top_triangle_count: top_count,
            bottom_triangle_count: bottom_count,
        }
    }

    fn carve_piece(
        mesh: &IndexedMesh,
        cx: f32,
        cy: f32,
        z_split: f32,
        floor_z: f32,
        radius: f32,
        chamber_height: f32,
        is_top: bool,
        config: &GrinderChamberConfig,
    ) -> IndexedMesh {
        let mut vertices: Vec<Vector<f32>> = Vec::new();
        let mut faces: Vec<IndexedTriangle> = Vec::new();

        // 1. Copy original vertices and triangles for the respective half (above or below Z split)
        let mut v_map = std::collections::HashMap::new();

        for face in &mesh.faces {
            let v0 = mesh.vertices[face.vertices[0]];
            let v1 = mesh.vertices[face.vertices[1]];
            let v2 = mesh.vertices[face.vertices[2]];

            let avg_z = (v0[2] + v1[2] + v2[2]) / 3.0;

            let in_half = if is_top { avg_z >= z_split } else { avg_z < z_split };

            if in_half {
                let mut new_indices = [0; 3];
                for (i, &old_v_idx) in face.vertices.iter().enumerate() {
                    let new_idx = *v_map.entry(old_v_idx).or_insert_with(|| {
                        let idx = vertices.len();
                        vertices.push(mesh.vertices[old_v_idx]);
                        idx
                    });
                    new_indices[i] = new_idx;
                }

                faces.push(IndexedTriangle {
                    normal: face.normal,
                    vertices: new_indices,
                });
            }
        }

        // 2. Carve Cylindrical Grinding Cavity Floor & Walls
        let segments = 32;
        let floor_v_start = vertices.len();

        // Cylindrical Chamber Wall Vertices
        for i in 0..segments {
            let angle = (i as f32 * 2.0 * PI) / (segments as f32);
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();

            vertices.push(Vector::new([x, y, z_split])); // 0: Wall at Z Split
            vertices.push(Vector::new([x, y, floor_z])); // 1: Wall at Floor
        }

        let dir = if is_top { 1.0 } else { -1.0 };

        for i in 0..segments {
            let next = (i + 1) % segments;
            let w1_split = floor_v_start + i * 2;
            let w1_floor = floor_v_start + i * 2 + 1;
            let w2_split = floor_v_start + next * 2;
            let w2_floor = floor_v_start + next * 2 + 1;

            // Internal Chamber Cylinder Wall Triangles
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, -dir]),
                vertices: [w1_split, w2_floor, w2_split],
            });
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, -dir]),
                vertices: [w1_split, w1_floor, w2_floor],
            });
        }

        // 3. Generate GIZEH Teeth & Central Magnet Hub on the Floor
        let half_height = chamber_height / 2.0;
        let effective_teeth_h = if config.recess_teeth {
            (half_height - config.teeth_recess_clearance_mm).max(1.0)
        } else {
            half_height * 0.95
        };

        TeethGenerator::generate_piece_teeth(
            cx,
            cy,
            floor_z,
            radius,
            effective_teeth_h,
            is_top,
            config.magnet_enabled,
            config.magnet_diameter_mm,
            config.magnet_depth_mm,
            &mut vertices,
            &mut faces,
        );

        // 4. Generate Interlocking Step Lip (Top) & Groove (Bottom)
        Self::generate_interlocking_step(cx, cy, z_split, radius, is_top, &mut vertices, &mut faces);

        IndexedMesh { vertices, faces }
    }

    /// Generates a cylindrical lip on the Top piece and a matching groove on the Bottom piece.
    fn generate_interlocking_step(
        cx: f32,
        cy: f32,
        z_split: f32,
        chamber_radius: f32,
        is_top: bool,
        vertices: &mut Vec<Vector<f32>>,
        faces: &mut Vec<IndexedTriangle>,
    ) {
        let lip_r_in = chamber_radius - 0.2;
        let lip_r_out = chamber_radius + 1.2;
        let lip_height = 2.0;

        let segments = 32;
        let start_v = vertices.len();
        let step_z = if is_top { z_split - lip_height } else { z_split + lip_height };

        for i in 0..segments {
            let angle = (i as f32 * 2.0 * PI) / (segments as f32);
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            vertices.push(Vector::new([cx + lip_r_in * cos_a, cy + lip_r_in * sin_a, z_split])); // 0: Base Inner
            vertices.push(Vector::new([cx + lip_r_out * cos_a, cy + lip_r_out * sin_a, z_split])); // 1: Base Outer
            vertices.push(Vector::new([cx + lip_r_in * cos_a, cy + lip_r_in * sin_a, step_z])); // 2: Step Inner
            vertices.push(Vector::new([cx + lip_r_out * cos_a, cy + lip_r_out * sin_a, step_z])); // 3: Step Outer
        }

        let dir = if is_top { -1.0 } else { 1.0 };

        for i in 0..segments {
            let next = (i + 1) % segments;
            let s_in1 = start_v + i * 4 + 2;
            let s_out1 = start_v + i * 4 + 3;

            let s_in2 = start_v + next * 4 + 2;
            let s_out2 = start_v + next * 4 + 3;

            // Step Lip Side Face
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, dir]),
                vertices: [s_in1, s_out2, s_out1],
            });
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, dir]),
                vertices: [s_in1, s_in2, s_out2],
            });
        }
    }
}
