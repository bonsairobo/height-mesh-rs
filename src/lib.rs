//! A small crate to generate a 3D mesh from a 2D heightmap.
//!
//! ```
//! use height_mesh::ndshape::{ConstShape, ConstShape2u32};
//! use height_mesh::{height_mesh, HeightMeshBuffer};
//!
//! // A 64^2 chunk with 1-pixel boundary padding.
//! type ChunkShape = ConstShape2u32<66, 66>;
//!
//! // This chunk will cover just a single quadrant of a paraboloid.
//! let mut height_map = [1.0; ChunkShape::SIZE as usize];
//! for i in 0u32..ChunkShape::SIZE {
//!     let [x, y] = ChunkShape::delinearize(i);
//!     height_map[i as usize] = ((x * x + y * y) as f32).sqrt();
//! }
//!
//! let mut buffer = HeightMeshBuffer::default();
//! height_mesh(&height_map, &ChunkShape {}, [0; 2], [65; 2], &mut buffer);
//!
//! // Some triangles were generated.
//! assert!(!buffer.indices.is_empty());
//! ```

pub use ndshape;

use ndshape::Shape;

/// The output buffers used by [`height_mesh`]. These buffers can be reused to avoid reallocating memory.
#[derive(Default)]
pub struct HeightMeshBuffer {
    /// The surface positions.
    pub positions: Vec<[f32; 3]>,
    /// The surface normals.
    ///
    /// The normals are **not** normalized, since that is done most efficiently on the GPU.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices, referring to offsets in the `positions` and `normals` vectors.
    pub indices: Vec<u32>,
    /// Used to map back from pixel stride to vertex index.
    pub stride_to_index: Vec<u32>,
}

impl HeightMeshBuffer {
    /// Clears all of the buffers, but keeps the memory allocated for reuse.
    pub fn reset(&mut self, array_size: usize) {
        self.positions.clear();
        self.normals.clear();
        self.indices.clear();

        // Just make sure this buffer is long enough, whether or not we've used it before.
        self.stride_to_index.resize(array_size, 0);
    }
}

/// Generates a mesh with a vertex at each point on the interior of `[min, max]`.
///
/// The generated vertices are of the form `[x, height, z]` where `height` is taken directly from `height_map`.
///
/// Surface normals are estimated using central differencing, which requires each vertex to have a complete Von Neumann
/// neighborhood. This means that points on the boundary are not eligible as mesh vertices, but they are still required.
///
/// This is illustrated in the ASCII art below, where "b" is a boundary point and "i" is an interior point. Line segments denote
/// the edges of the mesh.
///
/// ```text
/// b   b   b   b
///
/// b   i - i   b
///     | / |
/// b   i - i   b
///
/// b   b   b   b
/// ```
pub fn height_mesh<S: Shape<u32, 2>>(
    height_map: &[f32],
    map_shape: &S,
    min: [u32; 2],
    max: [u32; 2],
    output: &mut HeightMeshBuffer,
) {
    // SAFETY
    // Check the bounds on the array before we start using get_unchecked.
    assert!((map_shape.linearize(min) as usize) < height_map.len());
    assert!((map_shape.linearize(max) as usize) < height_map.len());

    output.reset(height_map.len());

    let [minx, miny] = min;
    let [maxx, maxy] = max;

    // Avoid accessing out of bounds with a 3x3x3 kernel.
    let iminx = minx + 1;
    let iminy = miny + 1;
    let imaxx = maxx - 1;
    let imaxy = maxy - 1;

    let x_stride = map_shape.linearize([1, 0]);
    let y_stride = map_shape.linearize([0, 1]);

    // Note: Although we use (x, y) for the coordinates of the height map, these should be considered (x, z) in world
    // coordinates, because +Y is the UP vector.
    for z in iminy..=imaxy {
        for x in iminx..=imaxx {
            let stride = map_shape.linearize([x, z]);
            let y = height_map[stride as usize];

            output.stride_to_index[stride as usize] = output.positions.len() as u32;
            output.positions.push([x as f32, y, z as f32]);

            // Use central differencing to calculate the surface normal.
            //
            // From calculus, we know that gradients are always orthogonal to a level set. The surface approximated by the
            // height map h(x, z) happens to be the 0 level set of the function:
            //
            // f(x, y, z) = y - h(x, z)
            //
            // And the gradient is:
            //
            // grad f = [-dh/dx, 1, -dh/dz]
            let l_stride = stride - x_stride;
            let r_stride = stride + x_stride;
            let b_stride = stride - y_stride;
            let t_stride = stride + y_stride;
            let l_y = unsafe { height_map.get_unchecked(l_stride as usize) };
            let r_y = unsafe { height_map.get_unchecked(r_stride as usize) };
            let b_y = unsafe { height_map.get_unchecked(b_stride as usize) };
            let t_y = unsafe { height_map.get_unchecked(t_stride as usize) };
            let dy_dx = (r_y - l_y) / 2.0;
            let dy_dz = (t_y - b_y) / 2.0;
            // Not normalized, because that's done more efficiently on the GPU.
            output.normals.push([-dy_dx, 1.0, -dy_dz]);
        }
    }

    // Only add a quad when p is the bottom-left corner of a quad that fits in the interior.
    let imaxx = imaxx - 1;
    let imaxy = imaxy - 1;

    for z in iminy..=imaxy {
        for x in iminx..=imaxx {
            let bl_stride = map_shape.linearize([x, z]);
            let br_stride = bl_stride + x_stride;
            let tl_stride = bl_stride + y_stride;
            let tr_stride = bl_stride + x_stride + y_stride;

            let bl_index = output.stride_to_index[bl_stride as usize];
            let br_index = output.stride_to_index[br_stride as usize];
            let tl_index = output.stride_to_index[tl_stride as usize];
            let tr_index = output.stride_to_index[tr_stride as usize];

            output
                .indices
                .extend_from_slice(&[bl_index, tl_index, tr_index, bl_index, tr_index, br_index]);
        }
    }
}
