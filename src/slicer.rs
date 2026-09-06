use stl_io::{IndexedMesh, Vector};

#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Segment2D {
    pub p1: Point2D,
    pub p2: Point2D,
}

#[derive(Debug, Clone)]
pub struct SingleLayerContour {
    pub points: Vec<Point2D>,
    pub is_closed: bool,
}

pub struct MeshSlicer;

impl MeshSlicer {
    /// Slices a 3D mesh at target Z height and produces 2D single-layer contours
    pub fn slice_at_z(mesh: &IndexedMesh, z_cut: f32) -> Vec<SingleLayerContour> {
        let mut segments = Vec::new();

        for face in &mesh.faces {
            let v1 = mesh.vertices[face.vertices[0]];
            let v2 = mesh.vertices[face.vertices[1]];
            let v3 = mesh.vertices[face.vertices[2]];

            if let Some(segment) = Self::intersect_triangle_z(v1, v2, v3, z_cut) {
                segments.push(segment);
            }
        }

        Self::stitch_segments(segments)
    }

    /// Intersects a 3D triangle with Z plane z = z_cut
    fn intersect_triangle_z(v1: Vector<f32>, v2: Vector<f32>, v3: Vector<f32>, z_cut: f32) -> Option<Segment2D> {
        let mut points = Vec::new();

        let edges = [(v1, v2), (v2, v3), (v3, v1)];
        for (a, b) in edges {
            if (a[2] <= z_cut && b[2] >= z_cut) || (b[2] <= z_cut && a[2] >= z_cut) {
                if (a[2] - b[2]).abs() > 1e-6 {
                    let t = (z_cut - a[2]) / (b[2] - a[2]);
                    let x = a[0] + t * (b[0] - a[0]);
                    let y = a[1] + t * (b[1] - a[1]);
                    points.push(Point2D { x, y });
                }
            }
        }

        if points.len() == 2 {
            Some(Segment2D {
                p1: points[0].clone(),
                p2: points[1].clone(),
            })
        } else {
            None
        }
    }

    /// Connects 2D segments into continuous closed contours
    fn stitch_segments(mut segments: Vec<Segment2D>) -> Vec<SingleLayerContour> {
        let mut contours = Vec::new();
        let eps = 1e-3;

        while !segments.is_empty() {
            let first = segments.remove(0);
            let mut contour_pts = vec![first.p1.clone(), first.p2.clone()];
            let mut added = true;

            while added {
                added = false;
                let mut i = 0;

                while i < segments.len() {
                    let last_pt = contour_pts.last().unwrap();
                    let seg = &segments[i];

                    let d1 = ((last_pt.x - seg.p1.x).powi(2) + (last_pt.y - seg.p1.y).powi(2)).sqrt();
                    let d2 = ((last_pt.x - seg.p2.x).powi(2) + (last_pt.y - seg.p2.y).powi(2)).sqrt();

                    if d1 < eps {
                        contour_pts.push(seg.p2.clone());
                        segments.remove(i);
                        added = true;
                    } else if d2 < eps {
                        contour_pts.push(seg.p1.clone());
                        segments.remove(i);
                        added = true;
                    } else {
                        i += 1;
                    }
                }
            }

            let first_pt = contour_pts.first().unwrap();
            let last_pt = contour_pts.last().unwrap();
            let dist_close = ((first_pt.x - last_pt.x).powi(2) + (first_pt.y - last_pt.y).powi(2)).sqrt();

            contours.push(SingleLayerContour {
                points: contour_pts,
                is_closed: dist_close < eps,
            });
        }

        contours
    }
}
