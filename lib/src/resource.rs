use std::env;
use std::path::{Path, PathBuf};

pub const RESOURCES_PATH: &str = "resources/";
pub fn resource(file: &str) -> PathBuf {
    let curr = env::current_exe().unwrap();
    // TODO: Recursively search for resources dir
    let resources = curr // phantoma/target/debug/___
        .parent()
        .unwrap() // phantoma/target/debug/
        .parent()
        .unwrap() // phantoma/target/
        .parent()
        .unwrap() // phantoma/
        .join(RESOURCES_PATH); // phantoma/resources/

    let path = Path::new(Path::new(file).file_name().unwrap());

    let dir = match path.extension() {
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
            "pkf" => "keyframes",
            "glb" => "scenes",
            ext => panic!("Unable to load format .{}!", ext),
        },
        None => panic!("Unable to determine resource type!"),
    };

    let path = resources.join(dir).join(path);

    if path.exists() {
        path
    } else {
        panic!("Resource {}/{} not found!", dir, file);
    }
}

pub fn read(file: &str) -> Vec<u8> {
    std::fs::read(resource(file)).unwrap()
}

pub fn read_str(file: &str) -> String {
    std::fs::read_to_string(resource(file)).unwrap()
}

pub fn read_buffered(file: &str) -> impl std::io::BufRead {
    let file = std::fs::File::open(resource(file)).unwrap();
    std::io::BufReader::new(file)
}

// TODO: Separate shader model creation from this method
pub fn read_shader(device: &wgpu::Device, file: &str) -> wgpu::ShaderModule {
    let data = read(file);
    let source = wgpu::util::make_spirv(&data);
    device.create_shader_module(source)
}

// pub fn read_scene(file: &str) -> gfx::Scene {

// }

// pub fn read_model(file: &str) -> Vec<ObjectData> {
//     let set = obj::parse(read_resource(file)).unwrap();
//     let mtl = mtl::parse(read_resource(&set.material_library.unwrap())).unwrap();

//     set.objects
//         .into_iter()
//         .map(|o| ObjectData::from(o, &mtl.materials))
//         .collect()
// }