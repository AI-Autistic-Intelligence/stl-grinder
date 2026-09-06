# ⚙️ `stl-grinder`

> An ultra-fast single-layer 3D mesh slicer, 2D contour generator, and single-layer STL extruder powered by **[Ferrox Framework](https://ferrox-rust.dev/)** in Rust.

---

## 🌟 Features

- ✂️ **Single-Layer Z-Slicing:** Slice any 3D STL mesh at a specific Z height plane and extract closed 2D contours in milliseconds.
- 📐 **SVG Vector Export:** Export 2D single-layer contours directly to scalable SVG vector files (`.svg`) for lasers, CNC, or 2D previewers.
- 🧊 **3D STL Single-Layer Extrusion:** Convert 2D contours into a printable 3D single-layer STL model with custom extrusion thickness (e.g. 0.2mm).
- 🌐 **Ferrox Framework API Server:** Launch a lightweight HTTP REST microservice (`stl-grinder serve`) powered by `ferrox-app` and `ferrox-transports` for remote web file processing (`POST /api/v1/grind`).

---

## 🚀 Usage

### 🛠️ Build
```bash
cargo build --release
```

### ⚙️ 1. Process an STL Model (CLI)
```bash
stl-grinder process model.stl --slice-z 0.2 --layer-height 0.2 --output-dir ./output
```

### 🌐 2. Launch Ferrox Framework API Microservice
```bash
stl-grinder serve --port 3000
```

#### API Endpoint:
- `POST /api/v1/grind` - Upload multipart `.stl` file for instant single-layer grinding & metadata calculation.

---

## 📜 License

Dual-licensed under `MIT` OR `Apache-2.0`. Created with ❤️ by **AI-Autistic-Intelligence**.
