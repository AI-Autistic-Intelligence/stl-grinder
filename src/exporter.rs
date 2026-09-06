use crate::slicer::SingleLayerContour;
use anyhow::Result;
use stl_io::{IndexedMesh, Triangle, Vector};
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;
use std::fs::File;
use std::path::Path as StdPath;

pub struct Exporter;

impl Exporter {
    /// Export 2D single-layer contours to SVG file
    pub fn export_svg<P: AsRef<StdPath>>(contours: &[SingleLayerContour], output_path: P) -> Result<()> {
        let mut document = Document::new().set("viewBox", (-150, -150, 300, 300));

        for contour in contours {
            if contour.points.is_empty() {
                continue;
            }

            let mut data = Data::new().move_to((contour.points[0].x, contour.points[0].y));
            for pt in &contour.points[1..] {
                data = data.line_to((pt.x, pt.y));
            }
            if contour.is_closed {
                data = data.close();
            }

            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "#FF5722")
                .set("stroke-width", 0.4)
                .set("d", data);

            document = document.add(path);
        }

        svg::save(output_path, &document)?;
        Ok(())
    }

    /// Extrudes 2D single-layer contours into a 3D printable single-layer STL file
    pub fn export_single_layer_stl<P: AsRef<StdPath>>(
        contours: &[SingleLayerContour],
        layer_height: f32,
        output_path: P,
    ) -> Result<()> {
        let mut vertices: Vec<Vector<f32>> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::new();

        for contour in contours {
            if contour.points.len() < 3 {
                continue;
            }

            let start_idx = vertices.len();
            let n = contour.points.len();

            // Bottom vertices (z = 0)
            for pt in &contour.points {
                vertices.push(Vector::new([pt.x, pt.y, 0.0]));
            }
            // Top vertices (z = layer_height)
            for pt in &contour.points {
                vertices.push(Vector::new([pt.x, pt.y, layer_height]));
            }

            // Side wall triangles
            for i in 0..n {
                let next = (i + 1) % n;

                let b1 = start_idx + i;
                let b2 = start_idx + next;
                let t1 = start_idx + n + i;
                let t2 = start_idx + n + next;

                triangles.push(Triangle {
                    normal: Vector::new([0.0, 0.0, 0.0]),
                    vertices: [b1, b2, t2],
                });
                triangles.push(Triangle {
                    normal: Vector::new([0.0, 0.0, 0.0]),
                    vertices: [b1, t2, t1],
                });
            }
        }

        let mesh = IndexedMesh {
            vertices,
            faces: triangles,
        };

        let mut out_file = File::create(output_path)?;
        stl_io::write_stl(&mut out_file, mesh.vertices.iter(), mesh.faces.iter())?;

        Ok(())
    }
}
