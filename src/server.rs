use crate::exporter::Exporter;
use crate::slicer::MeshSlicer;
use crate::stl_parser::StlLoader;
use axum::{
    extract::Multipart,
    routing::{get, post},
    Json, Router,
};
use ferrox_app::FerroxApp;
use ferrox_transports::http::HttpTransport;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::NamedTempFile;

pub async fn start_grinder_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/grind", post(grind_stl_mesh));

    let transport = HttpTransport::new(router, port);

    println!("⚡ Ferrox Framework HTTP Transport running Single-Layer Grinder on port {}...", port);

    FerroxApp::new()
        .add_transport(transport)
        .start()
        .await?;

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "stl-grinder-server",
        "framework": "Ferrox Framework v0.1.0",
        "author": "AI-Autistic-Intelligence"
    }))
}

async fn grind_stl_mesh(mut multipart: Multipart) -> Json<Value> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" || field.file_name().unwrap_or_default().ends_with(".stl") {
            if let Ok(bytes) = field.bytes().await {
                if let Ok(mut temp_file) = NamedTempFile::new() {
                    if temp_file.write_all(&bytes).is_ok() {
                        if let Ok((mesh, bbox)) = StlLoader::load(temp_file.path()) {
                            let cut_z = bbox.min_z + (bbox.height() * 0.2); // Slice at 20% height
                            let contours = MeshSlicer::slice_at_z(&mesh, cut_z);

                            let mut svg_out = NamedTempFile::new().unwrap();
                            let _ = Exporter::export_svg(&contours, svg_out.path());

                            return Json(json!({
                                "success": true,
                                "message": "STL mesh successfully ground into single layer contour",
                                "bounding_box": {
                                    "width_mm": bbox.width(),
                                    "depth_mm": bbox.depth(),
                                    "height_mm": bbox.height()
                                },
                                "slice_z_mm": cut_z,
                                "contours_count": contours.len(),
                                "points_count": contours.iter().map(|c| c.points.len()).sum::<usize>()
                            }));
                        }
                    }
                }
            }
        }
    }

    Json(json!({
        "success": false,
        "error": "Valid .stl file upload required in 'file' field"
    }))
}
