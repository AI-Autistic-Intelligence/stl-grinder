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

    /// Export an IndexedMesh directly to 3D STL file
    pub fn export_indexed_mesh_stl<P: AsRef<StdPath>>(mesh: &IndexedMesh, output_path: P) -> Result<()> {
        let mut triangles = Vec::with_capacity(mesh.faces.len());
        for f in &mesh.faces {
            let v0 = mesh.vertices[f.vertices[0]];
            let v1 = mesh.vertices[f.vertices[1]];
            let v2 = mesh.vertices[f.vertices[2]];
            triangles.push(Triangle {
                normal: f.normal,
                vertices: [v0, v1, v2],
            });
        }
        let mut out_file = File::create(output_path)?;
        stl_io::write_stl(&mut out_file, triangles.iter())?;
        Ok(())
    }

    /// Extrudes 2D single-layer contours into a 3D printable single-layer STL file
    pub fn export_single_layer_stl<P: AsRef<StdPath>>(
        contours: &[SingleLayerContour],
        layer_height: f32,
        output_path: P,
    ) -> Result<()> {
        let mut triangles: Vec<Triangle> = Vec::new();

        for contour in contours {
            if contour.points.len() < 3 {
                continue;
            }

            let n = contour.points.len();

            for i in 0..n {
                let next = (i + 1) % n;

                let pt1 = &contour.points[i];
                let pt2 = &contour.points[next];

                let b1 = Vector::new([pt1.x, pt1.y, 0.0]);
                let b2 = Vector::new([pt2.x, pt2.y, 0.0]);
                let t1 = Vector::new([pt1.x, pt1.y, layer_height]);
                let t2 = Vector::new([pt2.x, pt2.y, layer_height]);

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

        let mut out_file = File::create(output_path)?;
        stl_io::write_stl(&mut out_file, triangles.iter())?;

        Ok(())
    }
}
