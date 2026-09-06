# ⚙️ `stl-grinder`

> An ultra-fast single-layer 3D mesh slicer, 2D contour generator, interactive Web UI, and single-layer STL extruder powered by **[Ferrox Framework](https://ferrox-rust.dev/)** in Rust.

---

## 🌟 Features

- 🖥️ **Interactive Web UI:** Drag-and-drop 3D STL file upload with real-time 2D SVG contour rendering, Z-slice ratio slider, and layer thickness controls.
- ✂️ **Single-Layer Z-Slicing:** Slice any 3D STL mesh at a specific Z height plane and extract closed 2D contours in milliseconds.
- 📐 **SVG Vector Export:** Export 2D single-layer contours directly to scalable SVG vector files (`.svg`) for lasers, CNC, or 2D previewers.
- 🧊 **3D STL Single-Layer Extrusion:** Convert 2D contours into a printable 3D single-layer STL model with custom extrusion thickness (e.g. 0.2mm).
- 🌐 **Ferrox Framework API Server:** Launch a lightweight HTTP REST microservice (`stl-grinder serve`) powered by `ferrox-app` and `ferrox-transports` serving the embedded Web UI at `http://localhost:3000` and API endpoint `POST /api/v1/grind`.

---

## 🚀 Usage

### 🛠️ Build
```bash
cargo build --release
```

### 🖥️ 1. Launch Interactive Web UI & Ferrox HTTP Server
```bash
stl-grinder serve --port 3000
```
Open **`http://localhost:3000`** in your browser to drag-and-drop STL models and slice them visually!

### ⚙️ 2. Process an STL Model via CLI
```bash
stl-grinder process model.stl --slice-z 0.2 --layer-height 0.2 --output-dir ./output
```

---

## 📜 License

Dual-licensed under `MIT` OR `Apache-2.0`. Created with ❤️ by **AI-Autistic-Intelligence**.
