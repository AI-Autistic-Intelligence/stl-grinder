mod exporter;
mod grinder_analyzer;
mod mesh_carver;
mod server;
mod slicer;
mod stl_parser;
mod teeth_generator;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use exporter::Exporter;
use grinder_analyzer::MeshAnalyzer;
use mesh_carver::MeshCarver;
use server::start_grinder_server;
use slicer::MeshSlicer;
use stl_parser::StlLoader;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "stl-grinder",
    author = "AI-Autistic-Intelligence <info@ferrox-rust.dev>",
    version = "0.2.2",
    about = "⚙️ Ultra-fast 3D STL Grinder Carver, GIZEH Teeth Generator & Slicer powered by Ferrox Framework",
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
        /// Input .stl file path
        input: PathBuf,

        /// Slice Z ratio relative to model height (0.0 to 1.0)
        #[arg(short, long, default_value = "0.2")]
        z_ratio: f32,

        /// Output 3D single-layer STL thickness in mm
        #[arg(short, long, default_value = "0.2")]
        layer_height: f32,

        /// Output folder path
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },

    /// Carve a 3D STL model into a 2-piece 3D Grinder with GIZEH teeth and magnet pockets
    Carve {
        /// Input .stl file path
        input: PathBuf,

        /// Outer wall thickness in mm
        #[arg(short, long, default_value = "2.0")]
        wall_thickness: f32,

        /// Output directory path
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },

    /// Start a Ferrox Framework HTTP API server and open interactive Web UI in browser
    Serve {
        /// HTTP port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", "============================================================".bright_blue());
    println!("{}", "⚙️  Ferrox Framework - 3D STL Grinder Carver v0.2.2".bold().cyan());
    println!("{}", "============================================================".bright_blue());

    match &cli.command {
        Some(Commands::Process { input, z_ratio, layer_height, output }) => {
            let start = std::time::Instant::now();

            println!("📂 Loading STL model: {}", input.display().to_string().yellow());
            let (mesh, bbox) = StlLoader::load(input)?;

            let cut_z = bbox.min_z + (bbox.height() * z_ratio);
            println!("✂️ Slicing horizontal plane at Z = {:.2} mm (Ratio: {:.1}%)", cut_z, z_ratio * 100.0);

            let contours = MeshSlicer::slice_at_z(&mesh, cut_z);
            println!("✨ Generated {} single-layer contour(s)", contours.len().to_string().green());

            std::fs::create_dir_all(output)?;
            let stem = input.file_stem().unwrap_or_default().to_string_lossy();
            let svg_path = output.join(format!("{}_single_layer.svg", stem));
            let stl_path = output.join(format!("{}_single_layer.stl", stem));

            Exporter::export_svg(&contours, &svg_path)?;
            println!("💾 {:<24} {}", "Exported 2D SVG:".bold(), svg_path.display().to_string().cyan());

            Exporter::export_single_layer_stl(&contours, *layer_height, &stl_path)?;
            println!("💾 {:<24} {} (Thickness: {:.2} mm)", "Exported 3D STL:".bold(), stl_path.display().to_string().cyan(), layer_height);

            let elapsed = start.elapsed();
            println!("\n{}", "============================================================".bright_blue());
            println!("⚡ Processing completed in {:.2?}", elapsed);
            println!("============================================================\n");
        }

        Some(Commands::Carve { input, wall_thickness, output }) => {
            let start = std::time::Instant::now();
            println!("📂 Analyzing 3D STL model for optimal grinder chamber: {}", input.display().to_string().yellow());

            let (mesh, bbox) = StlLoader::load(input)?;
            let analysis = MeshAnalyzer::analyze(&mesh, &bbox, *wall_thickness);

            println!("🔍 Maximum Safe Chamber Diameter: {:.2} mm", (analysis.max_safe_chamber_radius_mm * 2.0).to_string().green());
            println!("✂️ Optimal Split Z Plane: {:.2} mm", analysis.optimal_z_split_mm.to_string().cyan());

            let carved = MeshCarver::carve_grinder_model(&mesh, &bbox, &analysis.suggested_config);

            std::fs::create_dir_all(output)?;
            let stem = input.file_stem().unwrap_or_default().to_string_lossy();
            let top_path = output.join(format!("{}_grinder_top.stl", stem));
            let bottom_path = output.join(format!("{}_grinder_bottom.stl", stem));

            Exporter::export_indexed_mesh_stl(&carved.top_mesh, &top_path)?;
            println!("💾 {:<24} {}", "Exported Top Part STL:".bold(), top_path.display().to_string().cyan());

            Exporter::export_indexed_mesh_stl(&carved.bottom_mesh, &bottom_path)?;
            println!("💾 {:<24} {}", "Exported Bottom Part STL:".bold(), bottom_path.display().to_string().cyan());

            let elapsed = start.elapsed();
            println!("\n{}", "============================================================".bright_blue());
            println!("⚡ 3D Grinder Carving completed in {:.2?}", elapsed);
            println!("============================================================\n");
        }

        Some(Commands::Serve { port }) => {
            println!("⚡ Ferrox Framework Web UI starting on port {}...", port);
            start_grinder_server(*port).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        None => {
            // Default when user double-clicks stl-grinder.exe!
            println!("⚡ Double-clicked! Starting Ferrox Web UI & opening browser...");
            start_grinder_server(3000).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}
