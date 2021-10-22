# height-mesh

A small crate to generate a 3D mesh from a 2D heightmap.

```rust
use height_mesh::ndshape::{ConstShape, ConstShape2u32};
use height_mesh::{height_mesh, HeightMeshBuffer};

// A 64^2 chunk with 1-pixel boundary padding.
type ChunkShape = ConstShape2u32<66, 66>;

// This chunk will cover just a single quadrant of a paraboloid.
let mut height_map = [1.0; ChunkShape::SIZE as usize];
for i in 0u32..ChunkShape::SIZE {
    let [x, y] = ChunkShape::delinearize(i);
    height_map[i as usize] = ((x * x + y * y) as f32).sqrt();
}

let mut buffer = HeightMeshBuffer::default();
height_mesh(&height_map, &ChunkShape {}, [0; 2], [65; 2], &mut buffer);

// Some triangles were generated.
assert!(!buffer.indices.is_empty());
```

License: MIT OR Apache-2.0
