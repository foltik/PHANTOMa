use super::pipeline::AnimatedMaterialDesc;

pub fn animations() -> Vec<AnimatedMaterialDesc> {
    let triopt = AnimatedMaterialDesc {
        name: "TriOpt",
        // nodes: vec!["Plane"],
        nodes: vec![],
        mats: vec!["_TriOpt"],
        images: vec![
            "P0021_.png",
            "P0021_1.png",
            "P0021_2.png",
            "P0021_3.png",
            "P0021_4.png",
            "P0021_5.png",
        ],
        fps: 6,
        unlit: false
    };

    vec![triopt]
}