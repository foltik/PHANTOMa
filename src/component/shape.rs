use rendy::mesh::{
    MeshBuilder, Normal, PosNormTangTex, PosNormTex, PosTex, Position, Tangent, TexCoord,
};

use genmesh::{
    generators::{self, IndexedPolygon, SharedVertex},
    EmitTriangles, MapVertex, Triangulate, Vertex, Vertices,
};

use nalgebra::Vector3;

pub enum Shape {
    /*Plane,
    Polygon,*/
    Cube,
    /*Sphere,
    IcoSphere,
    Cylinder,
    Torus,
    Cone*/
}

impl Shape {
    /// Generate `MeshBuilder` for the `Shape`
    pub fn generate<V>(&self, scale: Option<(f32, f32, f32)>) -> MeshBuilder<'static>
    where
        V: FromShape + Into<MeshBuilder<'static>>,
    {
        V::from(&self.generate_internal(scale)).into()
    }

    /// Generate vertices for the `Shape`
    #[allow(dead_code)]
    pub fn generate_vertices<V>(&self, scale: Option<(f32, f32, f32)>) -> V
    where
        V: FromShape,
    {
        V::from(&self.generate_internal(scale))
    }

    fn generate_internal(&self, scale: Option<(f32, f32, f32)>) -> InternalShape {
        let vertices = match *self {
            Shape::Cube => generate_vertices(generators::Cube::new(), scale),
        };
        InternalShape(vertices)
    }
}

fn generate_vertices<F, P, G>(
    generator: G,
    scale: Option<(f32, f32, f32)>,
) -> Vec<InternalVertexData>
where
    F: EmitTriangles<Vertex = Vertex>,
    F::Vertex: Clone + Copy + PartialEq,
    P: EmitTriangles<Vertex = usize>,
    G: SharedVertex<F::Vertex> + IndexedPolygon<P> + Iterator<Item = F>,
{
    let vertices = generator.shared_vertex_iter().collect::<Vec<_>>();
    generator
        .indexed_polygon_iter()
        .triangulate()
        .map(|f| {
            f.map_vertex(|u| {
                let v = vertices[u];
                let pos = scale
                    .map(|(x, y, z)| Vector3::new(v.pos.x * x, v.pos.y * y, v.pos.z * z))
                    .unwrap_or_else(|| Vector3::new(v.pos.x, v.pos.y, v.pos.z));
                let normal = scale
                    .map(|(x, y, z)| {
                        Vector3::new(v.normal.x * x, v.normal.y * y, v.normal.z * z).normalize()
                    })
                    .unwrap_or_else(|| Vector3::new(v.normal.x, v.normal.y, v.normal.z));
                let up = Vector3::y();
                let tangent = normal.cross(&up).cross(&normal);
                (
                    pos.into(),
                    normal.into(),
                    [(v.pos.x + 1.) / 2., (v.pos.y + 1.) / 2.],
                    tangent.into(),
                )
            })
        })
        .vertices()
        .collect::<Vec<_>>()
}

/// Vertex data for a basic shape.
pub type InternalVertexData = ([f32; 3], [f32; 3], [f32; 2], [f32; 3]);

/// Internal Shape, used for transformation from `genmesh` to `MeshBuilder`
#[derive(Debug)]
pub struct InternalShape(Vec<InternalVertexData>);

impl InternalShape {
    fn map_into<T, F: FnMut(&InternalVertexData) -> T>(&self, f: F) -> Vec<T> {
        self.0.iter().map(f).collect()
    }
}

/// Trait for providing conversion from a basic shape type.
pub trait FromShape {
    /// Convert from a shape to `Self` type.
    fn from(shape: &InternalShape) -> Self;
}

/// Internal trait for converting from vertex data to a shape type.
pub trait FromInternalVertex {
    /// Convert from a set of vertex data to `Self` type.
    fn from_internal(v: &InternalVertexData) -> Self;
}

impl<T: FromInternalVertex> FromShape for Vec<T> {
    fn from(shape: &InternalShape) -> Self {
        shape.map_into(T::from_internal)
    }
}

impl FromInternalVertex for Position {
    fn from_internal(v: &InternalVertexData) -> Self {
        Position([v.0[0], v.0[1], v.0[2]])
    }
}

impl FromInternalVertex for TexCoord {
    fn from_internal(v: &InternalVertexData) -> Self {
        TexCoord([v.2[0], v.2[1]])
    }
}

impl FromInternalVertex for Normal {
    fn from_internal(v: &InternalVertexData) -> Self {
        Normal([v.1[0], v.1[1], v.1[2]])
    }
}

impl FromInternalVertex for Tangent {
    fn from_internal(v: &InternalVertexData) -> Self {
        Tangent([v.3[0], v.3[1], v.3[2], 1.0])
    }
}

macro_rules! impl_interleaved {
    ($($type:ident { $($member:ident),*}),*,) => {
        $(impl FromInternalVertex for $type {
            fn from_internal(v: &InternalVertexData) -> Self {
                Self {
                    $($member: FromInternalVertex::from_internal(v),)*
                }
            }
        })*
    }
}

impl_interleaved! {
    PosTex { position, tex_coord },
    PosNormTex { position, normal, tex_coord },
    PosNormTangTex { position, normal, tangent, tex_coord },
}
