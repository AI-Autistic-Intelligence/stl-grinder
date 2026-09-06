mod exporter;
mod server;
mod slicer;
mod stl_parser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use exporter::Exporter;
use server::start_grinder_server;
use slicer::MeshSlicer;
use stl_parser::StlLoader;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "stl-grinder",
    author = "AI-Autistic-Intelligence <info@ferrox-rust.dev>",
    version = "0.2.1",
    about = "⚙️ Ultra-fast Single-Layer 3D Mesh Slicer, Contour Extractor & Extruder powered by Ferrox Framework",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Slice a 3D STL model into a 2D/3D single-layer contour & extruded model
    Process {
        /// Path to input 3D STL file
        file: PathBuf,

        /// Custom Z-height in mm to slice at (defaults to 20% of model height)
        #[arg(short = 'z', long)]
        slice_z: Option<f32>,

        /// Single-layer extrusion height in mm for exported 3D STL
        #[arg(short = 'l', long, default_value_t = 0.2)]
        layer_height: f32,

        /// Output directory for exported SVG & STL single-layer files
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },

    /// Start a Ferrox Framework HTTP API server and open interactive Web UI in browser
    Serve {
        /// Port to bind the Ferrox HTTP server to
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Process {
            file,
            slice_z,
            layer_height,
            output_dir,
        }) => {
            let start = std::time::Instant::now();
            println!("\n{}", "============================================================".bright_blue());
            println!("⚙️  {} {}", "Single-Layer Grinding:".bold().white(), file.display().to_string().yellow().bold());
            println!("{}\n", "============================================================".bright_blue());

            let (mesh, bbox) = StlLoader::load(&file)?;

            println!("📐  {:<24} {:.2} x {:.2} x {:.2} mm", "Bounding Box (WxDxH):".bold(), bbox.width(), bbox.depth(), bbox.height());
            println!("🔺  {:<24} {}", "Total Triangles:".bold(), mesh.faces.len());

            let cut_height = slice_z.unwrap_or_else(|| bbox.min_z + (bbox.height() * 0.2));
            println!("✂️   {:<24} {:.2} mm", "Target Z Slice Height:".bold(), cut_height);

            let contours = MeshSlicer::slice_at_z(&mesh, cut_height);
            println!("%  {:<24} {} contours", "Generated Single-Layer:".bold(), contours.len().to_string().green().bold());

            std::fs::create_dir_all(&output_dir)?;

            let stem = file.file_stem().unwrap_or_default().to_string_lossy();
            let svg_path = output_dir.join(format!("{}_single_layer.svg", stem));
            let stl_path = output_dir.join(format!("{}_single_layer.stl", stem));

            Exporter::export_svg(&contours, &svg_path)?;
            println!("💾  {:<24} {}", "Exported 2D SVG:".bold(), svg_path.display().to_string().cyan());

            Exporter::export_single_layer_stl(&contours, layer_height, &stl_path)?;
            println!("💾  {:<24} {} (Thickness: {:.2} mm)", "Exported 3D STL:".bold(), stl_path.display().to_string().cyan(), layer_height);

            let elapsed = start.elapsed();
            println!("\n{}", "============================================================".bright_blue());
            println!("⚡ Single-layer processing completed in {:.2?}", elapsed);
            println!("============================================================\n");
        }

        Some(Commands::Serve { port }) => {
            println!("⚡ Ferrox Framework Web UI starting on port {}...", port);
            start_grinder_server(port).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        None => {
            // Default when user double-clicks stl-grinder.exe!
            println!("⚡ Double-clicked! Starting Ferrox Web UI & opening browser...");
            start_grinder_server(3000).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}
