use stl_io::{IndexedTriangle, Vector};
use std::f32::consts::PI;



pub struct TeethGenerator;

impl TeethGenerator {
    /// Generates Gizeh-style trapezoidal/diamond wedge teeth for a given piece (Top or Bottom).
    /// `is_top`: if true, generates odd rings (1, 3, ...); if false, generates even rings (2, 4, ...).
    pub fn generate_piece_teeth(
        center_x: f32,
        center_y: f32,
        base_z: f32,
        chamber_radius: f32,
        teeth_height: f32,
        is_top: bool,
        magnet_enabled: bool,
        magnet_diameter: f32,
        magnet_depth: f32,
        vertices: &mut Vec<Vector<f32>>,
        faces: &mut Vec<IndexedTriangle>,
    ) {
        let direction = if is_top { -1.0 } else { 1.0 }; // Top teeth extend downwards (-Z), Bottom teeth extend upwards (+Z)
        let inner_hub_r = (magnet_diameter / 2.0 + 2.5).max(5.0);
        let outer_clearance_r = chamber_radius - 2.0;

        let available_radial_space = outer_clearance_r - inner_hub_r;
        let num_rings = 4;
        let ring_spacing = available_radial_space / (num_rings as f32);

        // Generate Central Hub
        Self::generate_central_hub(
            center_x,
            center_y,
            base_z,
            inner_hub_r,
            teeth_height * 0.9,
            direction,
            magnet_enabled,
            magnet_diameter,
            magnet_depth,
            vertices,
            faces,
        );

        // Generate Concentric Teeth Rings
        for ring_idx in 0..num_rings {
            let ring_is_top = ring_idx % 2 == 0;
            if ring_is_top != is_top {
                continue; // Skip rings assigned to the opposite piece
            }

            let r_mid = inner_hub_r + (ring_idx as f32 + 0.5) * ring_spacing;
            let teeth_count = 6 + ring_idx * 4; // Outer rings have more teeth
            let angle_step = 2.0 * PI / (teeth_count as f32);
            let tooth_width_r = ring_spacing * 0.35;

            for t_i in 0..teeth_count {
                let angle = t_i as f32 * angle_step;
                Self::generate_single_gizeh_tooth(
                    center_x,
                    center_y,
                    base_z,
                    r_mid,
                    angle,
                    tooth_width_r,
                    teeth_height,
                    direction,
                    vertices,
                    faces,
                );
            }
        }
    }

    /// Generates a central cylindrical hub with an optional magnet pocket hole (default 3.1mm x 2.1mm).
    fn generate_central_hub(
        cx: f32,
        cy: f32,
        base_z: f32,
        hub_r: f32,
        hub_h: f32,
        dir: f32,
        magnet_enabled: bool,
        magnet_d: f32,
        magnet_h: f32,
        vertices: &mut Vec<Vector<f32>>,
        faces: &mut Vec<IndexedTriangle>,
    ) {
        let segments = 16;
        let top_z = base_z + dir * hub_h;

        let mag_r = magnet_d / 2.0;
        let mag_bottom_z = top_z - dir * magnet_h;

        let start_v_idx = vertices.len();

        for i in 0..segments {
            let angle = (i as f32 * 2.0 * PI) / (segments as f32);
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Outer Hub Bottom
            vertices.push(Vector::new([cx + hub_r * cos_a, cy + hub_r * sin_a, base_z]));
            // Outer Hub Top
            vertices.push(Vector::new([cx + hub_r * cos_a, cy + hub_r * sin_a, top_z]));

            if magnet_enabled {
                // Magnet Hole Rim Top
                vertices.push(Vector::new([cx + mag_r * cos_a, cy + mag_r * sin_a, top_z]));
                // Magnet Hole Bottom
                vertices.push(Vector::new([cx + mag_r * cos_a, cy + mag_r * sin_a, mag_bottom_z]));
            }
        }

        let stride = if magnet_enabled { 4 } else { 2 };

        for i in 0..segments {
            let next = (i + 1) % segments;

            let b1 = start_v_idx + i * stride;
            let t1 = start_v_idx + i * stride + 1;
            let b2 = start_v_idx + next * stride;
            let t2 = start_v_idx + next * stride + 1;

            // Outer Hub Side Faces
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, dir]),
                vertices: [b1, t2, b2],
            });
            faces.push(IndexedTriangle {
                normal: Vector::new([0.0, 0.0, dir]),
                vertices: [b1, t1, t2],
            });

            if magnet_enabled {
                let mr1 = start_v_idx + i * stride + 2;
                let mb1 = start_v_idx + i * stride + 3;
                let mr2 = start_v_idx + next * stride + 2;
                let mb2 = start_v_idx + next * stride + 3;

                // Hub Top Surface (between hub_r and mag_r)
                faces.push(IndexedTriangle {
                    normal: Vector::new([0.0, 0.0, dir]),
                    vertices: [t1, mr2, t2],
                });
                faces.push(IndexedTriangle {
                    normal: Vector::new([0.0, 0.0, dir]),
                    vertices: [t1, mr1, mr2],
                });

                // Magnet Hole Inner Wall
                faces.push(IndexedTriangle {
                    normal: Vector::new([0.0, 0.0, -dir]),
                    vertices: [mr1, mb2, mr2],
                });
                faces.push(IndexedTriangle {
                    normal: Vector::new([0.0, 0.0, -dir]),
                    vertices: [mr1, mb1, mb2],
                });
            }
        }
    }

    /// Generates a single GIZEH-style diamond wedge tooth with 3D tapered angles.
    fn generate_single_gizeh_tooth(
        cx: f32,
        cy: f32,
        base_z: f32,
        r_mid: f32,
        angle: f32,
        r_width: f32,
        h: f32,
        dir: f32,
        vertices: &mut Vec<Vector<f32>>,
        faces: &mut Vec<IndexedTriangle>,
    ) {
        let r_in = r_mid - r_width;
        let r_out = r_mid + r_width;
        let angular_span = 0.25; // Radial span of tooth in radians

        let a1 = angle - angular_span / 2.0;
        let a2 = angle + angular_span / 2.0;
        let a_mid = angle;

        let top_z = base_z + dir * h;

        let v_start = vertices.len();

        // 4 Base Vertices (Bottom of tooth)
        vertices.push(Vector::new([cx + r_in * a1.cos(), cy + r_in * a1.sin(), base_z]));   // 0: Inner Left
        vertices.push(Vector::new([cx + r_out * a1.cos(), cy + r_out * a1.sin(), base_z])); // 1: Outer Left
        vertices.push(Vector::new([cx + r_out * a2.cos(), cy + r_out * a2.sin(), base_z])); // 2: Outer Right
        vertices.push(Vector::new([cx + r_in * a2.cos(), cy + r_in * a2.sin(), base_z]));   // 3: Inner Right

        // 2 Top Ridge Vertices (Diamond/Wedge Tip)
        let r_tip = r_mid;
        let tip_span = angular_span * 0.4;
        vertices.push(Vector::new([cx + r_tip * (a_mid - tip_span).cos(), cy + r_tip * (a_mid - tip_span).sin(), top_z])); // 4: Top Left
        vertices.push(Vector::new([cx + r_tip * (a_mid + tip_span).cos(), cy + r_tip * (a_mid + tip_span).sin(), top_z])); // 5: Top Right

        let b0 = v_start;
        let b1 = v_start + 1;
        let b2 = v_start + 2;
        let b3 = v_start + 3;
        let t4 = v_start + 4;
        let t5 = v_start + 5;

        // Side Sloped Faces of Diamond Wedge Tooth
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b0, b1, t4] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b1, b2, t5] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b1, t5, t4] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b2, b3, t5] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b3, b0, t4] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [b3, t4, t5] });
        faces.push(IndexedTriangle { normal: Vector::new([0.0, 0.0, dir]), vertices: [t4, t5, b2] });
    }
}
