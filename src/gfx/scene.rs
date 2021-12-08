use std::collections::HashMap;

use crate::math::prelude::*;

use super::frame::Frame;
use super::wgpu;

use super::camera::Camera;
use super::light::{LightDesc, Lights};
use super::material::MaterialDesc;
use super::mesh::{Mesh, MeshDesc};
use super::animation::Animation;

type Graph<T> = petgraph::stable_graph::StableDiGraph<T, ()>;
pub type NodeIndex = petgraph::graph::NodeIndex;
pub type MaterialIndex = usize;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translate: Vector3,
    pub rotate: Quat,
    pub scale: Vector3,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translate: Vector3::zero(),
            rotate: Quat::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl From<Transform> for Matrix4 {
    fn from(tr: Transform) -> Self {
        let t = Matrix4::from_translation(tr.translate);
        let r = Matrix4::from(tr.rotate);
        let s = Matrix4::from_nonuniform_scale(tr.scale.x, tr.scale.y, tr.scale.z);
        t * r * s
    }
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub transform: Transform,
}

pub struct SceneAnim {
    pub name: String,
    pub desc: Animation,
}

pub struct SceneMeshDesc {
    pub desc: MeshDesc,
    pub material: MaterialIndex,
    pub i: NodeIndex,
}

pub struct SceneLightDesc {
    pub desc: LightDesc,
    pub i: NodeIndex,
}

pub struct SceneCameraDesc {
    pub proj: Matrix4,
    pub i: NodeIndex,
}

#[derive(Default)]
pub struct SceneDesc {
    pub materials: Vec<MaterialDesc>,

    pub animations: Vec<SceneAnim>,
    pub lights: Vec<SceneLightDesc>,
    pub meshes: Vec<SceneMeshDesc>,
    pub camera: Option<SceneCameraDesc>,

    // skins: Vec<Skin>,
    pub nodes: Graph<Node>,
    pub root: NodeIndex,

    pub names: HashMap<String, NodeIndex>,
}

impl SceneDesc {
    pub fn flat(&self) -> HashMap<NodeIndex, Matrix4> {
        let mut nodes = self.nodes.clone();

        fn visit(
            map: &mut HashMap<NodeIndex, Matrix4>,
            nodes: &mut Graph<Node>,
            node: NodeIndex,
            parent: Option<NodeIndex>,
        ) {
            let mut transform = Matrix4::from(nodes[node].transform);
            if let Some(parent) = parent {
                transform = Matrix4::from(nodes[parent].transform) * transform;
            }
            map.insert(node, transform);

            let mut neighbors = nodes.neighbors_directed(node, petgraph::Outgoing).detach();
            while let Some((_, c)) = neighbors.next(nodes) {
                visit(map, nodes, c, Some(node))
            }
        }

        let mut map = HashMap::new();

        visit(&mut map, &mut nodes, self.root, None);
        map
    }
}

pub struct Scene {
    pub desc: SceneDesc,

    pub cam: Camera,
    pub cam_idx: NodeIndex,
    pub cam_layout: wgpu::BindGroupLayout,

    pub meshes: Vec<Mesh>,
    pub mesh_idxs: Vec<NodeIndex>,
    pub mesh_layout: wgpu::BindGroupLayout,

    pub lights: Lights,
    pub light_idxs: Vec<NodeIndex>,
    pub light_layout: wgpu::BindGroupLayout,
}

impl Scene {
    pub fn new(device: &wgpu::Device, desc: SceneDesc) -> Self {
        let transforms = desc.flat();

        let cam_layout = Camera::layout(device);
        let mesh_layout = Mesh::layout(device);
        let light_layout = Lights::layout(device);

        let cam_idx = desc.camera.as_ref().unwrap().i;
        let view = transforms[&cam_idx].invert().unwrap();
        let proj = &desc.camera.as_ref().unwrap().proj;
        let cam = Camera::new(device, &cam_layout, &view, proj);

        let meshes = desc
            .meshes
            .iter()
            .map(|m| Mesh::new(device, &mesh_layout, &m.desc, &transforms[&m.i]))
            .collect();
        let mesh_idxs = desc.meshes.iter().map(|m| m.i).collect();

        let (light_descs, light_idxs): (Vec<_>, Vec<_>) =
            desc.lights.iter().map(|l| (l.desc, l.i)).unzip();
        let light_transforms = light_idxs.iter().map(|i| view * transforms[i]).collect::<Vec<_>>();
        let lights = Lights::new(device, &light_layout, &light_descs, &light_transforms);

        Self {
            desc,

            cam,
            cam_idx,
            cam_layout,

            meshes,
            mesh_idxs,
            mesh_layout,

            lights,
            light_idxs,
            light_layout,
        }
    }

    pub fn update(&self, frame: &mut Frame) {
        let transforms = self.desc.flat();

        let view = transforms[&self.cam_idx].invert().unwrap();
        self.cam.view.upload(frame, &view);

        self.meshes
            .iter()
            .zip(self.mesh_idxs.iter())
            .for_each(|(m, i)| {
                m.transform.upload(frame, &transforms[i]);
            });

        let light_transforms = self.light_idxs.iter().map(|i| view * transforms[i]).collect::<Vec<_>>();

        self.lights.transforms.upload(frame, &light_transforms);
    }

    pub fn node(&self, name: &str) -> &Node {
        &self.desc.nodes[self.desc.names[name]]
    }

    pub fn node_mut(&mut self, name: &str) -> &mut Node {
        &mut self.desc.nodes[self.desc.names[name]]
    }
}
