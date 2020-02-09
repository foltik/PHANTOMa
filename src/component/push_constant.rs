use std::mem::size_of;

use nalgebra;

pub unsafe trait PushConstant {
    fn raw(&self) -> &[u32] where Self: Sized {
        let ptr = self as *const Self as *const u32;
        let sz = size_of::<Self>() / size_of::<u32>();
        unsafe {
            std::slice::from_raw_parts(ptr, sz)
        }
    }
}

unsafe impl PushConstant for nalgebra::Matrix4<f32> {}