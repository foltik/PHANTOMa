use crate::math::{self, prelude::*};

use super::scene::{
    Node, NodeIndex, SceneAnim, SceneCameraDesc, SceneDesc, SceneLightDesc, SceneMeshDesc,
    Transform,
};

use crate::gfx::animation::{self, Animation};
use crate::gfx::light::{LightDesc, LightType};
use crate::gfx::material::{MaterialAttr, MaterialDesc};
use crate::gfx::mesh::{MeshDesc, Vertex, VertexSkinned};

impl<'a> From<&gltf::Node<'a>> for Node {
    fn from(n: &gltf::Node<'a>) -> Self {
        let transform = match n.transform() {
            gltf::scene::Transform::Decomposed {
                translation: t,
                rotation: r,
                scale: s,
            } => Transform {
                translate: Vector3::new(t[0], t[1], t[2]),
                rotate: Quat::new(r[3], r[0], r[1], r[2]),
                scale: Vector3::new(s[0], s[1], s[2]),
            },
            _ => unimplemented!(),
        };

        Self {
            name: n.name().unwrap().to_owned(),
            transform,
        }
    }
}

impl From<gltf::Gltf> for SceneDesc {
    fn from(g: gltf::Gltf) -> Self {
        fn visit(data: &[u8], scene: &mut SceneDesc, n: gltf::Node, parent: NodeIndex) {
            let idx = scene.nodes.add_node((&n).into());
            scene.names.insert(n.name().unwrap().to_owned(), idx);
            scene.nodes.add_edge(parent, idx, ());
            log::trace!("Visiting '{}'", n.name().unwrap());

            if let Some(mesh) = n.mesh() {
                for p in mesh.primitives() {
                    let reader = p.reader(|_| Some(data));

                    let inds = reader.read_indices().unwrap().into_u32().collect();

                    let pos = reader
                        .read_positions()
                        .unwrap()
                        .map(|p| Vector3::new(p[0], p[1], p[2]));

                    let tex = reader
                        .read_tex_coords(0)
                        .unwrap()
                        .into_f32()
                        .map(|t| Vector2::new(t[0], t[1]));

                    let norm = reader
                        .read_normals()
                        .unwrap()
                        .map(|n| Vector3::new(n[0], n[1], n[2]));

                    let desc = if
                    /*p.get(&gltf::Semantic::Weights(0)).is_some()*/
                    false {
                        let joints =
                            reader.read_joints(0).unwrap().into_u16().map(|i| {
                                Vector4::new(i[0] as f32, i[1] as f32, i[2] as f32, i[3] as f32)
                            });

                        let weights = reader
                            .read_weights(0)
                            .unwrap()
                            .into_f32()
                            .map(|w| Vector4::new(w[0], w[1], w[2], w[3]));

                        let verts = pos
                            .zip(tex)
                            .zip(norm)
                            .zip(joints)
                            .zip(weights)
                            .map(|((((pos, tex), norm), joints), weights)| VertexSkinned {
                                pos,
                                tex,
                                norm,
                                joints,
                                weights,
                            })
                            .collect::<Vec<_>>();

                        MeshDesc::new(verts, inds)
                    } else {
                        let verts = pos
                            .zip(tex)
                            .zip(norm)
                            .map(|((pos, tex), norm)| Vertex { pos, tex, norm })
                            .collect::<Vec<_>>();

                        MeshDesc::new(verts, inds)
                    };

                    scene.meshes.push(SceneMeshDesc {
                        desc,
                        material: p.material().index().unwrap_or_else(|| {
                            panic!("Missing material for node {}", n.name().unwrap())
                        }),
                        i: idx,
                    });
                }
            }

            if let Some(light) = n.light() {
                use gltf::khr_lights_punctual::Kind;
                let desc = LightDesc {
                    ty: match light.kind() {
                        Kind::Directional => LightType::Directional,
                        Kind::Point => LightType::Point,
                        Kind::Spot { .. } => LightType::Spot,
                    } as u32,
                    color: light.color().into(),
                    intensity: light.intensity(),
                    range: light.range().unwrap_or(999999999.0),
                    angle: match light.kind() {
                        Kind::Spot {
                            outer_cone_angle, ..
                        } => outer_cone_angle,
                        _ => 0.0,
                    },
                    pad: 123.0,
                };

                scene.lights.push(SceneLightDesc { desc, i: idx });
            }

            if let Some(camera) = n.camera() {
                // Convert [-1, 1] depth to [0, 1]
                let correction = Matrix4::new(
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
                );

                use gltf::camera::Projection;
                let proj = correction
                    * match camera.projection() {
                        Projection::Orthographic(_) => unimplemented!("Ortho cameras unsupported"),
                        Projection::Perspective(p) => math::projection::perspective(
                            Rad(p.yfov()),
                            p.aspect_ratio().unwrap(),
                            p.znear(),
                            p.zfar().unwrap_or(10000.0),
                        ),
                    };

                if scene.camera.is_none() {
                    log::trace!("Main Camera: {:?}", camera.name().unwrap());
                    scene.camera = Some(SceneCameraDesc { proj, i: idx })
                }
            }

            for c in n.children() {
                visit(data, scene, c, idx);
            }
        }

        let data = g.blob.as_ref().unwrap();
        let mut scene = Self::default();

        for m in g.materials() {
            let pbr = m.pbr_metallic_roughness();
            scene.materials.push(MaterialDesc {
                name: m.name().unwrap().to_owned(),
                color: match pbr.base_color_texture() {
                    Some(tex) => {
                        let src = tex.texture().source().source();

                        use gltf::image::Source;
                        let (i, j) = match src {
                            Source::View { view, .. } => {
                                (view.offset(), view.offset() + view.length())
                            }
                            Source::Uri { .. } => unimplemented!("Only texture views supported"),
                        };

                        let img = image::load_from_memory(&data[i..j]).unwrap();

                        MaterialAttr::Map(img)
                    }
                    _ => MaterialAttr::Value(pbr.base_color_factor().into()),
                },
                emissive: m.emissive_factor().into(),
                unlit: m.unlit(),
            });
        }

        let root = scene.nodes.add_node(Node {
            name: "root".to_owned(),
            transform: Transform::identity(),
        });
        scene.root = root;

        for node in g.scenes().next().unwrap().nodes() {
            visit(data, &mut scene, node, root);
        }

        for a in g.animations() {
            let mut anim = Animation {
                name: a.name().unwrap().to_owned(),
                ..Animation::default()
            };

            for c in a.channels() {
                let target = c.target();
                let reader = c.reader(|_| Some(g.blob.as_ref().unwrap()));

                match reader.read_outputs().unwrap() {
                    gltf::animation::util::ReadOutputs::Translations(t) => anim.channels.push(
                        (animation::Channel::Translate(
                            t.map(|t| Vector3::new(t[0], t[1], t[2])).collect(),
                        ), scene.names[target.node().name().unwrap()]),
                    ),
                    gltf::animation::util::ReadOutputs::Rotations(r) => anim.channels.push(
                        (animation::Channel::Rotate(
                            r.into_f32().map(|r| Quat::new(r[3], r[0], r[1], r[2])).collect(),
                        ), scene.names[target.node().name().unwrap()]),
                    ),
                    gltf::animation::util::ReadOutputs::Scales(s) => anim.channels.push(
                        (animation::Channel::Scale(
                            s.map(|s| Vector3::new(s[0], s[1], s[2])).collect(),
                        ), scene.names[target.node().name().unwrap()]),
                    ),
                    _ => unimplemented!("No morph target weights!"),
                }
            }

            anim.len = anim.channels.iter().max_by_key(|c| c.0.len()).unwrap().0.len();

            scene.animations.push(SceneAnim {
                name: a.name().unwrap().to_owned(),
                desc: anim,
            });
        }

        scene
    }
}

pub fn parse(data: &[u8]) -> SceneDesc {
    let g = gltf::Gltf::from_slice(data).unwrap();
    debug(&g);

    g.into()
}

fn debug_print(n: &gltf::Node, depth: usize) {
    let pad = " ".repeat(depth * 4);

    log::trace!(
        "{}Node: {}",
        pad,
        n.name().unwrap(),
    );

    // if let Some(m) = n.mesh() {
    //     log::trace!("{} Mesh: {}", pad, m.name().unwrap());
    // }

    if let Some(l) = n.light() {
        log::trace!("{} Light: {}", pad, l.name().unwrap());
        log::trace!("{}  Color: {:?}", pad, l.color());
        log::trace!("{}  Intensity: {:?}", pad, l.intensity());
        log::trace!("{}  Range: {:?}", pad, l.range());
    }

    if let Some(c) = n.camera() {
        log::trace!("{} Camera: {}", pad, c.name().unwrap());
        if let gltf::camera::Projection::Perspective(p) = c.projection() {
            log::trace!("{}  Aspect: {:?}", pad, p.aspect_ratio());
            log::trace!("{}  Y FoV: {}", pad, p.yfov());
            log::trace!("{}  Near: {}", pad, p.znear());
            log::trace!("{}  Far: {:?}", pad, p.zfar());
        }
    }
}

fn debug_explore(n: &gltf::Node, depth: usize) {
    debug_print(n, depth);
    for c in n.children() {
        debug_explore(&c, depth + 1);
    }
}

fn debug(g: &gltf::Gltf) {
    // for b in g.buffers() {
    //     log::trace!("Buffer {}[{}]", b.index(), b.length());
    // }
    // log::trace!("");

    for m in g.materials() {
        let i = m.pbr_metallic_roughness();
        log::trace!("Material {}: {}", m.index().unwrap(), m.name().unwrap());
        log::trace!(" Base Color Factor: {:?}", i.base_color_factor());
        // log::trace!(" Base Color Texture: {:?}", i.base_color_texture());
        // log::trace!(" Emissive Factor: {:?}", m.emissive_factor());
        // log::trace!(" Emissive Texture: {:?}", m.emissive_texture());
        log::trace!(" Unlit: {}", m.unlit());
    }
    log::trace!("");

    for a in g.animations() {
        log::trace!("Animation {}: {}", a.index(), a.name().unwrap());
        for c in a.channels() {
            let mut n = 0;
            let reader = c.reader(|_| Some(g.blob.as_ref().unwrap()));
            match reader.read_outputs().unwrap() {
                gltf::animation::util::ReadOutputs::Translations(t) => {
                    for _tr in t {
                        // log::trace!("  Translations: {:?}", tr);
                        n += 1;
                    }
                }
                gltf::animation::util::ReadOutputs::Rotations(r) => {
                    for _rot in r.into_f32() {
                        // log::trace!("  Rotations: {:?}", rot);
                        n += 1;
                    }
                }
                _ => {}
            }
            log::trace!(
                " Channel: {} -> {:?} {:?} x{}",
                c.target().node().name().unwrap(),
                c.target().property(),
                c.sampler().interpolation(),
                n,
            );
        }
    }
    log::trace!("");

    for s in g.scenes() {
        for n in s.nodes() {
            debug_explore(&n, 0);
        }
    }
}
