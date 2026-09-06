use crate::exporter::Exporter;
use crate::grinder_analyzer::{GrinderChamberConfig, MeshAnalyzer};
use crate::mesh_carver::MeshCarver;
use crate::slicer::MeshSlicer;
use crate::stl_parser::StlLoader;
use axum::{
    extract::{DefaultBodyLimit, Multipart},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::{json, Value};
use std::io::Write;
use tempfile::NamedTempFile;
use tower_http::cors::CorsLayer;

pub async fn start_grinder_server(preferred_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .route("/", get(serve_web_ui))
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/grind", post(grind_stl_mesh))
        .route("/api/v1/analyze", post(analyze_stl_mesh))
        .route("/api/v1/carve_grinder", post(carve_grinder_3d))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // Support up to 100MB STL files
        .layer(CorsLayer::permissive());

    let mut ports_to_try = vec![preferred_port, 8080, 8081, 8082, 8083, 8084, 8085, 3000];
    ports_to_try.dedup();

    let mut listener = None;
    let mut bound_port = 0;

    for &port in &ports_to_try {
        let addr = format!("127.0.0.1:{}", port);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => {
                listener = Some(l);
                bound_port = port;
                break;
            }
            Err(_) => {
                // If port is occupied, check if a server is active on it
                if let Ok(socket_addr) = addr.parse::<std::net::SocketAddr>() {
                    if std::net::TcpStream::connect_timeout(&socket_addr, std::time::Duration::from_millis(200)).is_ok() {
                        println!("⚡ Ferrox Web UI is already active on http://127.0.0.1:{}", port);
                        open_app_window(port);
                        return Ok(());
                    }
                }
            }
        }
    }

    let listener = match listener {
        Some(l) => l,
        None => return Err("Could not bind to any HTTP port in range 8080-8085".into()),
    };

    let url = format!("http://127.0.0.1:{}", bound_port);
    println!("⚡ Launching Ferrox Framework HTTP Transport (Localhost Loopback Only) on {}...", url);
    println!("🌐 Interactive Web UI live at: {}", url);

    open_app_window(bound_port);

    axum::serve(listener, router).await?;

    Ok(())
}

fn find_standalone_browser() -> Option<std::path::PathBuf> {
    let candidates = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];

    for path in &candidates {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let local_candidates = [
            format!(r"{}\Google\Chrome\Application\chrome.exe", local_app_data),
            format!(r"{}\Microsoft\Edge\Application\msedge.exe", local_app_data),
        ];
        for path in &local_candidates {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(r"C:\Program Files (x86)\Microsoft\EdgeCore") {
        for entry in entries.flatten() {
            let exe = entry.path().join("msedge.exe");
            if exe.exists() {
                return Some(exe);
            }
        }
    }

    None
}

fn open_app_window(port: u16) {
    let url = format!("http://127.0.0.1:{}", port);
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        let app_arg = format!("--app={}", url);
        let profile_dir = std::env::temp_dir().join("stl-grinder-browser-profile");
        let user_data_arg = format!("--user-data-dir={}", profile_dir.to_string_lossy());

        if let Some(browser_path) = find_standalone_browser() {
            println!("🚀 Launching Standalone App Window mode via: {:?}", browser_path);
            let child = std::process::Command::new(browser_path)
                .arg(&app_arg)
                .arg(&user_data_arg)
                .arg("--name=3D STL Grinder Carver")
                .spawn();

            if let Ok(mut child_proc) = child {
                let _ = child_proc.wait();
                println!("👋 Standalone App Window closed by user. Terminating stl-grinder backend...");
                std::process::exit(0);
            }
        } else {
            let _ = webbrowser::open(&url);
        }
    });
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

async fn analyze_stl_mesh(mut multipart: Multipart) -> Json<Value> {
    let mut stl_bytes: Option<Vec<u8>> = None;
    let mut wall_thickness: f32 = 2.0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "wall_thickness" {
            if let Ok(text) = field.text().await {
                wall_thickness = text.parse().unwrap_or(2.0);
            }
        } else if name == "file" || field.file_name().unwrap_or_default().ends_with(".stl") {
            if let Ok(bytes) = field.bytes().await {
                stl_bytes = Some(bytes.to_vec());
            }
        }
    }

    if let Some(bytes) = stl_bytes {
        if let Ok(mut temp_file) = NamedTempFile::new() {
            if temp_file.write_all(&bytes).is_ok() {
                if let Ok((mesh, bbox)) = StlLoader::load(temp_file.path()) {
                    let analysis = MeshAnalyzer::analyze(&mesh, &bbox, wall_thickness);
                    return Json(json!({
                        "success": true,
                        "analysis": analysis
                    }));
                }
            }
        }
    }

    Json(json!({
        "success": false,
        "error": "Valid .stl file upload required in 'file' field"
    }))
}

async fn carve_grinder_3d(mut multipart: Multipart) -> Json<Value> {
    let mut stl_bytes: Option<Vec<u8>> = None;
    let mut z_split: Option<f32> = None;
    let mut chamber_radius: f32 = 20.0;
    let mut chamber_height: f32 = 10.0;
    let mut wall_thickness: f32 = 2.0;
    let mut magnet_enabled: bool = true;
    let mut magnet_diameter: f32 = 3.1;
    let mut magnet_depth: f32 = 2.1;
    let mut recess_teeth: bool = true;
    let mut recess_clearance: f32 = 0.5;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "z_split" {
            if let Ok(text) = field.text().await {
                z_split = text.parse().ok();
            }
        } else if name == "chamber_radius" {
            if let Ok(text) = field.text().await {
                chamber_radius = text.parse().unwrap_or(20.0);
            }
        } else if name == "chamber_height" {
            if let Ok(text) = field.text().await {
                chamber_height = text.parse().unwrap_or(10.0);
            }
        } else if name == "wall_thickness" {
            if let Ok(text) = field.text().await {
                wall_thickness = text.parse().unwrap_or(2.0);
            }
        } else if name == "magnet_enabled" {
            if let Ok(text) = field.text().await {
                magnet_enabled = text.parse().unwrap_or(true);
            }
        } else if name == "magnet_diameter" {
            if let Ok(text) = field.text().await {
                magnet_diameter = text.parse().unwrap_or(3.1);
            }
        } else if name == "magnet_depth" {
            if let Ok(text) = field.text().await {
                magnet_depth = text.parse().unwrap_or(2.1);
            }
        } else if name == "recess_teeth" {
            if let Ok(text) = field.text().await {
                recess_teeth = text.parse().unwrap_or(true);
            }
        } else if name == "recess_clearance" {
            if let Ok(text) = field.text().await {
                recess_clearance = text.parse().unwrap_or(0.5);
            }
        } else if name == "file" || field.file_name().unwrap_or_default().ends_with(".stl") {
            if let Ok(bytes) = field.bytes().await {
                stl_bytes = Some(bytes.to_vec());
            }
        }
    }

    if let Some(bytes) = stl_bytes {
        if let Ok(mut temp_file) = NamedTempFile::new() {
            if temp_file.write_all(&bytes).is_ok() {
                if let Ok((mesh, bbox)) = StlLoader::load(temp_file.path()) {
                    let analysis = MeshAnalyzer::analyze(&mesh, &bbox, wall_thickness);
                    let split_height = z_split.unwrap_or(analysis.optimal_z_split_mm);

                    let config = GrinderChamberConfig {
                        z_split_mm: split_height,
                        chamber_radius_mm: chamber_radius.min(analysis.max_safe_chamber_radius_mm),
                        max_allowed_radius_mm: analysis.max_safe_chamber_radius_mm,
                        chamber_height_mm: chamber_height,
                        wall_thickness_mm: wall_thickness,
                        magnet_enabled,
                        magnet_diameter_mm: magnet_diameter,
                        magnet_depth_mm: magnet_depth,
                        teeth_rings_count: 4,
                        recess_teeth,
                        teeth_recess_clearance_mm: recess_clearance,
                    };

                    let carved = MeshCarver::carve_grinder_model(&mesh, &bbox, &config);

                    // Generate Top, Bottom, Combined STL bytes
                    let top_temp = NamedTempFile::new().unwrap();
                    let bottom_temp = NamedTempFile::new().unwrap();
                    let combined_temp = NamedTempFile::new().unwrap();

                    let _ = Exporter::export_indexed_mesh_stl(&carved.top_mesh, top_temp.path());
                    let _ = Exporter::export_indexed_mesh_stl(&carved.bottom_mesh, bottom_temp.path());
                    let _ = Exporter::export_indexed_mesh_stl(&carved.combined_mesh, combined_temp.path());

                    let top_bytes = std::fs::read(top_temp.path()).unwrap_or_default();
                    let bottom_bytes = std::fs::read(bottom_temp.path()).unwrap_or_default();
                    let combined_bytes = std::fs::read(combined_temp.path()).unwrap_or_default();

                    let top_b64 = BASE64.encode(&top_bytes);
                    let bottom_b64 = BASE64.encode(&bottom_bytes);
                    let combined_b64 = BASE64.encode(&combined_bytes);
                    let original_b64 = BASE64.encode(&bytes);

                    return Json(json!({
                        "success": true,
                        "message": "3D STL Grinder model successfully carved with GIZEH teeth and magnet pockets",
                        "config": config,
                        "top_stl_b64": top_b64,
                        "bottom_stl_b64": bottom_b64,
                        "combined_stl_b64": combined_b64,
                        "original_stl_b64": original_b64,
                        "top_triangles": carved.top_triangle_count,
                        "bottom_triangles": carved.bottom_triangle_count
                    }));
                }
            }
        }
    }

    Json(json!({
        "success": false,
        "error": "Valid .stl file upload required in 'file' field"
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
