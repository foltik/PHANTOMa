use std::env;
use std::path::{Path, PathBuf};

pub const RESOURCES_PATH: &'static str = "resources/";
pub fn resource(file: &str) -> PathBuf {
    let curr = env::current_exe().unwrap();
    // TODO: Recursively search for resources dir
    let resources = curr // phantoma/sketches/___/target/debug/___
        .parent()
        .unwrap() // sketches/___/target/debug/
        .parent()
        .unwrap() // sketches/___/target/
        .parent()
        .unwrap() // sketches/___/
        //.parent().unwrap() // sketches/
        //.parent().unwrap() // /
        .join(RESOURCES_PATH); // sketches/resources/
    let file = Path::new(Path::new(file).file_name().unwrap());

    let dir = match file.extension() {
        Some(os) => match os.to_str().unwrap() {
            "txt" => "lorem",
            "ttf" => "fonts",
            "spv" => "shaders",
            "obj" => "models",
            "mtl" => "models",
            "dds" => "textures",
            "png" => "textures",
            "jpg" => "textures",
            "tga" => "textures",
            "ckf" => "keyframes",
            _ => "",
        },
        None => "",
    };

    resources.join(dir).join(file)
}

pub fn read_resource(file: &str) -> String {
    std::fs::read_to_string(resource(file)).expect(&format!("{} not found", file))
}

pub fn read_resource_raw(file: &str) -> Vec<u8> {
    std::fs::read(resource(file)).expect(&format!("{} not found", file))
}

pub fn read_resource_buffered(file: &str) -> impl std::io::BufRead {
    let file = std::fs::File::open(resource(file)).expect(&format!("{} not found", file));
    std::io::BufReader::new(file)
}

// pub fn read_model(file: &str) -> Vec<ObjectData> {
//     let set = obj::parse(read_resource(file)).unwrap();
//     let mtl = mtl::parse(read_resource(&set.material_library.unwrap())).unwrap();

//     set.objects
//         .into_iter()
//         .map(|o| ObjectData::from(o, &mtl.materials))
//         .collect()
// }

// TODO: put this in gfx?
pub fn read_shader(device: &wgpu::Device, file: &str) -> wgpu::ShaderModule {
    let data = read_resource_raw(file);
    let source = wgpu::util::make_spirv(&data);
    device.create_shader_module(source)
}