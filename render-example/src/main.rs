use height_mesh::ndshape::{ConstShape, ConstShape2u32};
use height_mesh::{height_mesh, HeightMeshBuffer};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
        wireframe::{WireframeConfig, WireframePlugin},
    },
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use obj_exporter::{export_to_file, Geometry, ObjSet, Object, Primitive, Shape, Vertex};
use std::f32::consts::PI;

fn main() {
    App::build()
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                // The Wireframe requires NonFillPolygonMode feature
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = true;

    let (buffer, mesh) = heightmap_to_mesh(&mut meshes, |p| 10.0 * sine2d(5.0, p));

    spawn_pbr(
        &mut commands,
        &mut materials,
        mesh,
        Transform::from_translation(Vec3::new(-32.0, 0.0, -32.0)),
    );

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 50.0, 50.0)),
        light: Light {
            range: 200.0,
            intensity: 8000.0,
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 75.0, 50.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    write_mesh_to_obj_file(&buffer);
}

fn heightmap_to_mesh(
    meshes: &mut Assets<Mesh>,
    heightmap: impl Fn([f32; 2]) -> f32,
) -> (HeightMeshBuffer, Handle<Mesh>) {
    type SampleShape = ConstShape2u32<66, 66>;

    let mut samples = [0.0; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(64, SampleShape::delinearize(i));
        samples[i as usize] = heightmap(p);
    }

    let mut buffer = HeightMeshBuffer::default();
    height_mesh(&samples, &SampleShape {}, [0; 2], [65; 2], &mut buffer);

    let num_vertices = buffer.positions.len();

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.set_attribute(
        "Vertex_Position",
        VertexAttributeValues::Float3(buffer.positions.clone()),
    );
    render_mesh.set_attribute(
        "Vertex_Normal",
        VertexAttributeValues::Float3(buffer.normals.clone()),
    );
    render_mesh.set_attribute(
        "Vertex_Uv",
        VertexAttributeValues::Float2(vec![[0.0; 2]; num_vertices]),
    );
    render_mesh.set_indices(Some(Indices::U32(buffer.indices.clone())));

    (buffer, meshes.add(render_mesh))
}

fn spawn_pbr(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: Handle<Mesh>,
    transform: Transform,
) {
    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.roughness = 0.9;

    commands.spawn_bundle(PbrBundle {
        mesh,
        material: materials.add(material),
        transform,
        ..Default::default()
    });
}

fn write_mesh_to_obj_file(buffer: &HeightMeshBuffer) {
    export_to_file(
        &ObjSet {
            material_library: None,
            objects: vec![Object {
                name: "mesh".to_string(),
                vertices: buffer
                    .positions
                    .iter()
                    .map(|&[x, y, z]| Vertex {
                        x: x as f64,
                        y: y as f64,
                        z: z as f64,
                    })
                    .collect(),
                normals: buffer
                    .normals
                    .iter()
                    .map(|&[x, y, z]| Vertex {
                        x: x as f64,
                        y: y as f64,
                        z: z as f64,
                    })
                    .collect(),
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: buffer
                        .indices
                        .chunks(3)
                        .map(|tri| Shape {
                            primitive: Primitive::Triangle(
                                (tri[0] as usize, None, Some(tri[0] as usize)),
                                (tri[1] as usize, None, Some(tri[1] as usize)),
                                (tri[2] as usize, None, Some(tri[2] as usize)),
                            ),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
                tex_vertices: vec![],
            }],
        },
        "mesh.obj",
    )
    .unwrap();
}

fn sine2d(n: f32, [x, y]: [f32; 2]) -> f32 {
    ((x / 2.0) * n * PI).sin() + ((y / 2.0) * n * PI).sin()
}

fn into_domain(array_dim: u32, [x, y]: [u32; 2]) -> [f32; 2] {
    [
        (2.0 * x as f32 / array_dim as f32) - 1.0,
        (2.0 * y as f32 / array_dim as f32) - 1.0,
    ]
}
