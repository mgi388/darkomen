use bevy_math::prelude::*;
use bevy_render::{
    mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
    prelude::*,
    render_asset::RenderAssetUsages,
    render_resource::VertexFormat,
};

use crate::m3d::{Object, ObjectFlags};

pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 988540918, VertexFormat::Uint32);

pub(super) fn mesh_from_m3d_object(object: &Object) -> Mesh {
    let mut translation = Vec3::default();

    if object
        .flags
        .contains(ObjectFlags::CUSTOM_TRANSLATION_ENABLED)
    {
        translation = object.translation;
    }

    let mut positions: Vec<[f32; 3]> = Default::default();
    let mut uv0s: Vec<[f32; 2]> = Default::default();
    let mut normals: Vec<[f32; 3]> = Default::default();
    let mut colors: Vec<[f32; 4]> = Default::default();
    let mut texture_indices: Vec<u32> = Default::default();
    let mut indices: Vec<u32> = Default::default();

    let mut vertex_index = 0;
    for face in object.faces.clone().iter_mut() {
        face.indices.reverse();

        for index in face.indices.iter() {
            let vertex = object.vertices.get(*index as usize).unwrap();

            positions.push([
                vertex.position.z + translation.z,
                vertex.position.y + translation.y,
                vertex.position.x + translation.x,
            ]);

            uv0s.push([vertex.uv.x, vertex.uv.y]);

            normals.push([vertex.normal.z, vertex.normal.y, vertex.normal.x]);

            // When using vertex colors, if they are black, nothing is rendered.
            colors.push([1.0, 1.0, 1.0, 1.0]);

            texture_indices.push(face.texture_index as u32);

            indices.push(vertex_index);

            vertex_index += 1;
        }
    }

    // Add UV1s for lightmaps.
    //
    // TODO: Should we only do this for the objects that need lightmaps?

    // Find bounds for the XZ projection.
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for vertex in object.vertices.iter() {
        // TODO: Not sure if adding translation is necessary here or if it is
        // pointless due to the UV1s being normalized.
        let pos = Vec3::new(
            vertex.position.z + translation.z,
            vertex.position.y + translation.y,
            vertex.position.x + translation.x,
        );
        min_x = min_x.min(pos.x);
        max_x = max_x.max(pos.x);
        min_z = min_z.min(pos.z);
        max_z = max_z.max(pos.z);
    }

    let size_x = max_x - min_x;
    let size_z = max_z - min_z;

    // Calculate UV1s based on the bounds.
    let mut uv1s: Vec<[f32; 2]> = Default::default();
    for face in object.faces.clone().iter_mut() {
        // Not sure why, but the face indices need to be un-reversed for the
        // UV1s to work for Bevy's lightmap.
        face.indices.reverse();

        for &index in face.indices.iter() {
            let vertex = &object.vertices[index as usize];

            // TODO: Not sure if adding translation is necessary here or if it
            // is pointless due to the UV1s being normalized.
            let pos = Vec3::new(
                vertex.position.z + translation.z,
                vertex.position.y + translation.y,
                vertex.position.x + translation.x,
            );

            // Map X and Z to [0, 1] range.
            let u = (pos.x - min_x) / size_x;
            let v = (pos.z - min_z) / size_z;
            uv1s.push([u, v]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv0s)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, uv1s)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indices)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use bevy_render::{
        mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
        prelude::*,
    };

    use crate::m3d::{Face, Object, ObjectFlags, Vertex};

    use super::*;

    fn assert_float32x3_attribute(
        mesh: &Mesh,
        attr: MeshVertexAttribute,
        expected: &Vec<[f32; 3]>,
    ) {
        if let Some(VertexAttributeValues::Float32x3(values)) = mesh.attribute(attr.clone()) {
            assert_eq!(values, expected);
        } else {
            panic!("Mesh does not have a {} attribute", attr.name);
        }
    }

    fn assert_float32x2_attribute(
        mesh: &Mesh,
        attr: MeshVertexAttribute,
        expected: &Vec<[f32; 2]>,
    ) {
        if let Some(VertexAttributeValues::Float32x2(values)) = mesh.attribute(attr.clone()) {
            assert_eq!(values, expected);
        } else {
            panic!("Mesh does not have a {} attribute", attr.name);
        }
    }

    fn assert_float32x4_attribute(
        mesh: &Mesh,
        attr: MeshVertexAttribute,
        expected: &Vec<[f32; 4]>,
    ) {
        if let Some(VertexAttributeValues::Float32x4(values)) = mesh.attribute(attr.clone()) {
            assert_eq!(values, expected);
        } else {
            panic!("Mesh does not have a {} attribute", attr.name);
        }
    }

    fn assert_uint32_attribute(mesh: &Mesh, attr: MeshVertexAttribute, expected: &Vec<u32>) {
        if let Some(VertexAttributeValues::Uint32(values)) = mesh.attribute(attr.clone()) {
            assert_eq!(values, expected);
        } else {
            panic!("Mesh does not have a {} attribute", attr.name);
        }
    }

    fn assert_indices(mesh: &Mesh, expected: &Vec<u32>) {
        if let Some(Indices::U32(values)) = mesh.indices() {
            assert_eq!(values, expected);
        } else {
            panic!("Mesh does not have indices");
        }
    }

    fn create_test_object() -> Object {
        let vertices = vec![
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.25),
                uv: Vec2::new(0.2, 0.3),
                normal: Vec3::new(0.1, 0.4, 0.7),
                ..Default::default()
            },
            Vertex {
                position: Vec3::new(1.5, -1.0, 0.75),
                uv: Vec2::new(0.6, 0.8),
                normal: Vec3::new(0.3, 0.5, 0.9),
                ..Default::default()
            },
            Vertex {
                position: Vec3::new(0.75, 1.25, -0.5),
                uv: Vec2::new(0.4, 0.1),
                normal: Vec3::new(0.2, 0.6, 0.8),
                ..Default::default()
            },
        ];

        let faces = vec![Face {
            indices: [0, 1, 2],
            texture_index: 1,
            ..Default::default()
        }];

        Object {
            vertices,
            faces,
            ..Default::default()
        }
    }

    #[test]
    fn test_mesh_from_m3d_object_simple() {
        let object = create_test_object();
        let mesh = mesh_from_m3d_object(&object);

        assert_float32x3_attribute(
            &mesh,
            Mesh::ATTRIBUTE_POSITION,
            // Faces are in reverse order. M3D's X and Z are swapped to match
            // Bevy's coordinate system.
            &vec![[-0.5, 1.25, 0.75], [0.75, -1.0, 1.5], [0.25, -0.5, 0.5]],
        );
        assert_float32x2_attribute(
            &mesh,
            Mesh::ATTRIBUTE_UV_0,
            // Faces are in reverse order.
            &vec![[0.4, 0.1], [0.6, 0.8], [0.2, 0.3]],
        );
        assert_float32x2_attribute(
            &mesh,
            Mesh::ATTRIBUTE_UV_1,
            // Faces are in original order (first reverse is undone). Normalized
            // to [0, 1] range.
            &vec![[0.0, 0.25], [1.0, 1.0], [0.6, 0.0]],
        );
        assert_float32x3_attribute(
            &mesh,
            Mesh::ATTRIBUTE_NORMAL,
            // Faces are in reverse order. M3D's X and Z are swapped to match
            // Bevy's coordinate system.
            &vec![[0.8, 0.6, 0.2], [0.9, 0.5, 0.3], [0.7, 0.4, 0.1]],
        );
        assert_float32x4_attribute(
            &mesh,
            Mesh::ATTRIBUTE_COLOR,
            // Color is white otherwise nothing is rendered.
            &vec![
                [1.0, 1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
            ],
        );
        assert_uint32_attribute(&mesh, ATTRIBUTE_TEXTURE_INDEX, &vec![1, 1, 1]);
        assert_indices(&mesh, &vec![0, 1, 2]);
    }

    #[test]
    fn test_mesh_from_m3d_object_with_translation() {
        let mut object = create_test_object();
        object.flags |= ObjectFlags::CUSTOM_TRANSLATION_ENABLED;
        object.translation = Vec3::new(1.0, 2.0, 3.0);

        let mesh = mesh_from_m3d_object(&object);

        // Check if the translation is applied to the positions.
        assert_float32x3_attribute(
            &mesh,
            Mesh::ATTRIBUTE_POSITION,
            // Faces are in reverse order. M3D's X and Z are swapped to match
            // Bevy's coordinate system.
            &vec![
                [-0.5 + 3.0, 1.25 + 2.0, 0.75 + 1.0],
                [0.75 + 3.0, -1.0 + 2.0, 1.5 + 1.0],
                [0.25 + 3.0, -0.5 + 2.0, 0.5 + 1.0],
            ],
        );
        assert_float32x2_attribute(
            &mesh,
            Mesh::ATTRIBUTE_UV_0,
            // Faces are in reverse order.
            &vec![[0.4, 0.1], [0.6, 0.8], [0.2, 0.3]],
        );
        assert_float32x2_attribute(
            &mesh,
            Mesh::ATTRIBUTE_UV_1,
            // Faces are in original order (first reverse is undone). Normalized
            // to [0, 1] range.
            &vec![[0.0, 0.25], [1.0, 1.0], [0.6, 0.0]],
        );
    }

    #[test]
    fn test_mesh_from_m3d_object_empty_object() {
        let object = Object::default();

        let mesh = mesh_from_m3d_object(&object);

        assert_float32x3_attribute(&mesh, Mesh::ATTRIBUTE_POSITION, &vec![] as &Vec<[f32; 3]>);
        assert_float32x2_attribute(&mesh, Mesh::ATTRIBUTE_UV_0, &vec![] as &Vec<[f32; 2]>);
        assert_float32x2_attribute(&mesh, Mesh::ATTRIBUTE_UV_1, &vec![] as &Vec<[f32; 2]>);
        assert_float32x3_attribute(&mesh, Mesh::ATTRIBUTE_NORMAL, &vec![] as &Vec<[f32; 3]>);
        assert_float32x4_attribute(&mesh, Mesh::ATTRIBUTE_COLOR, &vec![] as &Vec<[f32; 4]>);
        assert_uint32_attribute(&mesh, ATTRIBUTE_TEXTURE_INDEX, &vec![] as &Vec<u32>);
        assert_indices(&mesh, &vec![]);
    }
}
