use crate::exporter::Exporter;
use crate::slicer::MeshSlicer;
use crate::stl_parser::StlLoader;
use axum::{
    extract::{DefaultBodyLimit, Multipart},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use ferrox_app::FerroxApp;
use ferrox_transports::http::HttpTransport;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::NamedTempFile;
use tower_http::cors::CorsLayer;

pub async fn start_grinder_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .route("/", get(serve_web_ui))
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/grind", post(grind_stl_mesh))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // Support up to 100MB STL files
        .layer(CorsLayer::permissive());

    let transport = HttpTransport::new(router, port);

    let url = format!("http://localhost:{}", port);
    println!("⚡ Launching Ferrox Framework HTTP Transport for Single-Layer Grinder on {}...", url);
    println!("🌐 Interactive Web UI live at: {}", url);

    // Auto-open default browser for standard users
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let _ = webbrowser::open(&url);
    });

    FerroxApp::new()
        .add_transport(transport)
        .start()
        .await?;

    Ok(())
}

async fn serve_web_ui() -> Html<&'static str> {
    Html(include_str!("web_ui.html"))
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
    let mut z_ratio: f32 = 0.2;
    let mut _layer_height: f32 = 0.2;
    let mut stl_bytes: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "z_ratio" {
            if let Ok(text) = field.text().await {
                z_ratio = text.parse().unwrap_or(0.2);
            }
        } else if name == "layer_height" {
            if let Ok(text) = field.text().await {
                _layer_height = text.parse().unwrap_or(0.2);
            }
        } else if name == "file" || field.file_name().unwrap_or_default().ends_with(".stl") {
            if let Ok(bytes) = field.bytes().await {
                stl_bytes = Some(bytes.to_vec());
            }
        }
    }

    if let Some(bytes) = stl_bytes {
        println!("📥 Received STL file payload: {} bytes", bytes.len());
        if let Ok(mut temp_file) = NamedTempFile::new() {
            if temp_file.write_all(&bytes).is_ok() {
                match StlLoader::load(temp_file.path()) {
                    Ok((mesh, bbox)) => {
                        let cut_z = bbox.min_z + (bbox.height() * z_ratio);
                        let contours = MeshSlicer::slice_at_z(&mesh, cut_z);

                        // Generate SVG string
                        let svg_file = NamedTempFile::new().unwrap();
                        let _ = Exporter::export_svg(&contours, svg_file.path());
                        let svg_content = std::fs::read_to_string(svg_file.path()).unwrap_or_default();

                        println!("⚡ Slice complete: {} contours generated at Z={:.2}mm", contours.len(), cut_z);

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
                            "points_count": contours.iter().map(|c| c.points.len()).sum::<usize>(),
                            "svg_raw": svg_content
                        }));
                    }
                    Err(e) => {
                        println!("❌ Error parsing STL file: {}", e);
                        return Json(json!({
                            "success": false,
                            "error": format!("Failed to parse STL mesh: {}", e)
                        }));
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
