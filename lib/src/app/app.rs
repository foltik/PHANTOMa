use crate::window::{Window, Proxy};

pub struct App {
    proxy: Proxy,

    pub window: Window,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    //draw_state: DrawState,
    //pub(crate) ui: ui::Arrangement,
    //pub mouse: state::Mouse,
    //pub keys: state::Keys,

    pub frame: u64,
    pub t: f32,
    pub dt: f32,
}

impl App {
    pub const ASSETS_DIRECTORY_NAME: &'static str = "assets";

    // Create a new `App`.
    pub(crate) fn new(
        proxy: Proxy,
        window: Window,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        // let ui = ui::Arrangement::new();
        // let mouse = state::Mouse::new();
        // let keys = state::Keys::default();
        let app = App {
            proxy,
            window,
            instance,
            adapter,
            device,
            queue,

            // focused_window,
            // adapters,
            // windows,
            // draw_state,
            // ui,
            // mouse,
            // keys,
            // time,
            frame: 0,
            t: 0.0,
            dt: 0.0,
        };
        app
    }

    // /// Find and return the absolute path to the project's `assets` directory.
    // ///
    // /// This method looks for the assets directory in the following order:
    // ///
    // /// 1. Checks the same directory as the executable.
    // /// 2. Recursively checks exe's parent directories (to a max depth of 5).
    // /// 3. Recursively checks exe's children directories (to a max depth of 3).
    // pub fn assets_path(&self) -> Result<PathBuf, find_folder::Error> {
    //     find_assets_path()
    // }

    // /// The path to the current project directory.
    // ///
    // /// The current project directory is considered to be the directory containing the cargo
    // /// manifest (aka the `Cargo.toml` file).
    // ///
    // /// **Note:** Be careful not to rely on this directory for apps or sketches that you wish to
    // /// distribute! This directory is mostly useful for local sketches, experiments and testing.
    // pub fn project_path(&self) -> Result<PathBuf, find_folder::Error> {
    //     find_project_path()
    // }

    /// A handle to the **App** that can be shared across threads.
    ///
    /// This can be used to "wake up" the **App**'s inner event loop.
    pub fn create_proxy(&self) -> Proxy {
        self.proxy.clone()
    }

    pub fn fps(&self) -> f32 {
        1000.0 / self.dt
    }
}
