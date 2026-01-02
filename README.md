# tinyrenderer-rs

A tiny **software rasterizer written in Rust**, built from scratch by following and adapting the
[**TinyRenderer**](https://haqr.eu/tinyrenderer/) tutorial (originally in C++).
The goal is to deeply understand the **entire 3D rendering pipeline** — without GPU APIs,
drivers, or hidden magic.

This is a **learning project**, not a production renderer.

---

## Goals

- Learn how a modern raster pipeline actually works
- Reimplement the TinyRenderer tutorial in **idiomatic Rust**
- Improve Rust skills (ownership, lifetimes, performance-aware code)
- Keep everything small, readable, and debuggable

---

## What’s Implemented (or Planned)

### Core Pipeline

- [x] Framebuffer abstraction
- [x] Line drawing
- [x] Triangle rasterization (barycentric coordinates)
- [x] Z-buffer (depth testing)
- [ ] Backface culling
- [ ] Viewport, projection, and camera transforms
- [ ] Near-plane clipping

### Assets & Data

- [ ] OBJ model loading
- [ ] Texture loading
- [ ] UV interpolation
- [ ] Perspective-correct interpolation

### Shading

- [ ] Flat shading
- [ ] Lambert (diffuse) lighting
- [ ] Specular highlights
- [ ] Normal interpolation
- [ ] Shader-like pipeline stages

### Extras (Stretch Goals)

- [ ] Wireframe / debug views
- [ ] MSAA
- [ ] Shadow mapping
- [ ] Normal mapping
- [ ] CPU performance optimizations
