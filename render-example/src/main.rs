use std::f32::consts::PI;

use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, WireframeConfig}, render::{render_resource::PrimitiveTopology, mesh::{Indices, VertexAttributeValues}}};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use height_mesh::{HeightMeshBuffer, ndshape::{ConstShape2u32, ConstShape}, height_mesh};
use obj_exporter::*;



pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "heightmapper".to_string(),
                ..default()
                },
            ..default()
        }))
        .insert_resource(ClearColor(Color::GRAY))
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 120.0, // default: 12.0
        })
        .add_plugin(NoCameraPlayerPlugin)
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = true;

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
    

    let (buffer, mesh) = heightmap_to_mesh(&mut meshes, |p| 10.0 * sine2d(5.0, p));

    spawn_pbr(
        &mut commands,
        &mut materials,
        mesh,
        Transform::from_translation(Vec3::new(-32.0, 0.0, -32.0)),
    );

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(FlyCam);

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
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        buffer.positions.clone(),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        buffer.normals.clone(),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0; 2]; num_vertices],
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


    commands.spawn(PbrBundle {
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