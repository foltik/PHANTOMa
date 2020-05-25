use nannou::math::cgmath::Vector3;

use obj::{IndexTuple, Material, Obj, ObjData, ObjMaterial, Object, SimplePolygon};

use super::gfx::material::{MaterialAttrDesc, MaterialDesc};
use super::gfx::model::Vertex;
use crate as lib;

pub fn read_obj(file: &str) -> ObjData {
    let mut obj = Obj::load(lib::resource(file)).expect(&format!("{} not found", file));

    // Fix up coordinates from blender export
    for p in &mut obj.data.position {
        p[0] = -p[0];
    }

    for uv in &mut obj.data.texture {
        uv[1] = -uv[1];
    }

    for n in &mut obj.data.normal {
        n[0] = -n[0];
    }

    obj.load_mtls_fn(|_, mtl| Ok(lib::read_resource_buffered(mtl)))
        .unwrap();

    obj.data
}

pub fn vertices(obj: &ObjData, mesh: &Object) -> Vec<Vertex> {
    let mut verts = Vec::new();

    for SimplePolygon(vs) in &mesh.groups[0].polys {
        assert_eq!(
            vs.len(),
            3,
            "obj loader encountered non-triangle primitive!"
        );

        let remap = |p: &IndexTuple| {
            (
                obj.position[p.0],
                obj.texture[p.1.unwrap()],
                obj.normal[p.2.unwrap()],
            )
        };
        verts.push(remap(&vs[0]));
        verts.push(remap(&vs[2]));
        verts.push(remap(&vs[1]));
    }

    verts
}

pub fn material(obj: &ObjData, mesh: &Object) -> MaterialDesc {
    if let ObjMaterial::Mtl(mat) = &mesh.groups[0].material.as_ref().unwrap() {
        let attr = |p: &Option<[f32; 3]>, map: &Option<String>| {
            let p = p.unwrap_or_else(|| [0.0f32; 3]);
            MaterialAttrDesc {
                col: Vector3::new(p[0], p[1], p[2]),
                map: map.clone(),
            }
        };

        MaterialDesc {
            ambient: attr(&mat.ka, &mat.map_ka),
            diffuse: attr(&mat.kd, &mat.map_kd),
            specular: attr(&mat.ks, &mat.map_ks),
            emissive: attr(&mat.ke, &mat.map_ke),
            alpha: mat.d.unwrap_or_else(|| 1.0),
        }
    } else {
        panic!("");
    }
}
