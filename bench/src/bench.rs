use height_mesh::ndshape::{ConstShape, ConstShape2u32};
use height_mesh::{height_mesh, HeightMeshBuffer};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::f32::consts::PI;

type SampleShape = ConstShape2u32<66, 66>;

fn bench_sine2d(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_sine2d");
    let mut samples = [0.0; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(64, SampleShape::delinearize(i));
        samples[i as usize] = sine2d(5.0, p);
    }

    // Do a single run first to allocate the buffer to the right size.
    let mut buffer = HeightMeshBuffer::default();
    height_mesh(&samples, &SampleShape {}, [0; 2], [65; 2], &mut buffer);
    let num_triangles = buffer.indices.len() / 3;

    group.bench_with_input(
        BenchmarkId::from_parameter(format!("tris={}", num_triangles)),
        &(),
        |b, _| {
            b.iter(|| height_mesh(&samples, &SampleShape {}, [0; 2], [65; 2], &mut buffer));
        },
    );
    group.finish();
}

criterion_group!(benches, bench_sine2d);
criterion_main!(benches);

fn sine2d(n: f32, [x, y]: [f32; 2]) -> f32 {
    ((x / 2.0) * n * PI).sin() + ((y / 2.0) * n * PI).sin()
}

fn into_domain(array_dim: u32, [x, y]: [u32; 2]) -> [f32; 2] {
    [
        (2.0 * x as f32 / array_dim as f32) - 1.0,
        (2.0 * y as f32 / array_dim as f32) - 1.0,
    ]
}
