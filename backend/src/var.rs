#[cfg(target_os = "windows")]
pub const IMAGE_ROOT_PATH: &str = r"E:\tt";

#[cfg(not(target_os = "windows"))]
pub const IMAGE_ROOT_PATH: &str = "/home/songday/website/blog/upload/image";

// 这里由于 len() 是 const fn，所以可以被调用
pub const IMAGE_ROOT_PATH_LENGTH: usize = IMAGE_ROOT_PATH.len();
pub const BLOG_PAGE_SIZE: u8 = 20u8;
pub const I64SIZE: usize = std::mem::size_of::<i64>();
