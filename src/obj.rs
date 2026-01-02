use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn load_obj(
    filename: &PathBuf,
) -> Result<(Vec<(f32, f32, f32)>, Vec<[usize; 3]>), std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut vertices: Vec<(f32, f32, f32)> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" if parts.len() >= 4 => {
                // Vertex: v x y z
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                vertices.push((x, y, z));
            }
            "f" if parts.len() >= 4 => {
                // Face: f v1 v2 v3 (vertices are 1-indexed)
                // Note: faces can be "v" or "v/vt" or "v/vt/vn" format
                let parse_vertex_index = |s: &str| -> usize {
                    let idx: i32 = s.split('/').next().unwrap_or("0").parse().unwrap_or(0);
                    (idx - 1).max(0) as usize
                };

                let v1 = parse_vertex_index(parts[1]);
                let v2 = parse_vertex_index(parts[2]);
                let v3 = parse_vertex_index(parts[3]);
                faces.push([v1, v2, v3]);
            }
            _ => {} // Ignore other lines
        }
    }

    Ok((vertices, faces))
}
