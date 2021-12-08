use std::env;
use std::path::{Path, PathBuf};

pub const RESOURCES_PATH: &str = "resources/";
pub fn resource(file: &str) -> PathBuf {
    let mut curr = env::current_exe().unwrap();

    // Find the target dir that the executable is running from
    while curr.file_name().unwrap() != "target" {
        if !curr.pop() {
            panic!("Failed to find resources directory.");
        }
    }

    // TODO: Better way of specifying resources dir
    let resources = curr // repo/target/
        .parent()
        .unwrap() // repo/
        .join(RESOURCES_PATH); // repo/resources/

    let path = Path::new(Path::new(file).file_name().unwrap());

    let dir = match path.extension() {
        Some(os) => match os.to_str().unwrap() {
            "txt" => "lorem",
            "ttf" => "fonts",
            "otf" => "fonts",
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
    // FIXME: Need shader validation
    // let source = wgpu::ShaderSource::SpirV(wgpu::util::make_spirv_raw(&data));
    let source = wgpu::util::make_spirv_raw(&data);
    unsafe { 
        device.create_shader_module_spirv(&::wgpu::ShaderModuleDescriptorSpirV {
            label: None,
            source,
        })
        // device.create_shader_module_unchecked(&wgpu::ShaderModuleDescriptor {
        //     label: None,
        //     source,
        // })
    }
}

pub fn read_gltf(file: &str) -> crate::gfx::scene::SceneDesc {
    let data = read(file);

    let before_read = std::time::Instant::now();
    let gltf = crate::gfx::gltf::parse(&data);
    log::trace!("Loaded {} in {:?}", file, before_read.elapsed());

    gltf
}

pub fn read_image(file: &str) -> image::DynamicImage {
    let data = read(file);

    let start = std::time::Instant::now();
    let img = image::load_from_memory(&data).unwrap();
    log::trace!("Loaded {} in {:?}", file, start.elapsed());

    img
}

pub fn read_scene(device: &wgpu::Device, file: &str) -> crate::gfx::scene::Scene {
    let gltf = read_gltf(file);
    crate::gfx::scene::Scene::new(device, gltf)
}

pub fn read_font(file: &str) -> wgpu_glyph::ab_glyph::FontArc {
    wgpu_glyph::ab_glyph::FontArc::try_from_vec(read(file)).unwrap()
}

// pub fn read_model(file: &str) -> Vec<ObjectData> {
//     let set = obj::parse(read_resource(file)).unwrap();
//     let mtl = mtl::parse(read_resource(&set.material_library.unwrap())).unwrap();

//     set.objects
//         .into_iter()
//         .map(|o| ObjectData::from(o, &mtl.materials))
//         .collect()
// }