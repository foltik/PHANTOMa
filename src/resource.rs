use std::path::Path;
pub use rust_embed::RustEmbed as Embed;

use crate::config::Config;

#[derive(Embed)]
#[folder = "resources/"]
struct Resources;

pub fn dir(file: &str) -> String {
    let path = Path::new(Path::new(file).file_name().unwrap());

    let ext = path.extension().unwrap().to_str().unwrap();
    let dir = match ext {
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
        "dem" => "demos",
        "cfg"  => "config",
        _     => "other",
    };

    Path::new(dir).join(path).into_os_string().into_string().unwrap()
}

pub fn dir_live(file: &str) -> String {
    let curr = std::env::current_exe().unwrap();
    // TODO: Recursively search for resources dir
    let resources = curr // phantoma/target/debug/___
        .parent()
        .unwrap() // phantoma/target/debug/
        .parent()
        .unwrap() // phantoma/target/
        .parent()
        .unwrap() // phantoma/
        .join("resources/"); // phantoma/resources/

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
            "dem" => "demos",
            "cfg"  => "config",
            ext => panic!("Unable to load format .{}!", ext),
        },
        None => panic!("Unable to determine resource type!"),
    };

    let path = resources.join(dir).join(path);

    if path.exists() {
        let str = path.into_os_string().into_string().unwrap();
        log::debug!("Loading {:?}", str);
        str
    } else {
        panic!("Resource {}/{} not found!", dir, file);
    }
}


// pub fn read(file: &str) -> Vec<u8> {
//     std::fs::read(dir_live(file)).unwrap()
// }

use std::borrow::Cow;
pub fn read(file: &str) -> Cow<'static, [u8]> {
    Resources::get(&dir(file)).expect(&format!("resource '{}' not found", file)).data
}

pub fn read_str(file: &str) -> String {
    String::from_utf8(read(file).to_vec()).unwrap()
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
    wgpu_glyph::ab_glyph::FontArc::try_from_vec(read(file).to_vec()).unwrap()
}

pub fn read_cfg(file: &str) -> Config {
    Config::from(&read_str(file))
}

// pub fn read_model(file: &str) -> Vec<ObjectData> {
//     let set = obj::parse(read_resource(file)).unwrap();
//     let mtl = mtl::parse(read_resource(&set.material_library.unwrap())).unwrap();

//     set.objects
//         .into_iter()
//         .map(|o| ObjectData::from(o, &mtl.materials))
//         .collect()
// }
