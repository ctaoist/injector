//! 向程序注入 dll
//!
//! 只支持 windows 系统
//!
//! ## Examples
//!
//! ```rsut
//! use injector::{get_process_id, inject_dll, eject_dll};
//!
//! let pid = get_process_id("WeChat.exe\0");
//! inject_dll(pid, "C:\dllTest.dll");
//! ```

mod utils;
pub use utils::{get_process_id, open_process};

mod inject;

/// 向进程中注入 dll
pub fn inject_dll(pid: u32, dll_path: &str) {
  if !utils::file_exist(dll_path) {
    log::error!("{} 不存在", dll_path);
    return;
  }

  inject::inject_dll(pid, dll_path);
}

/// 卸载已注入的 dll
pub fn eject_dll(pid: u32, dll_path: &str) {
  inject::eject_dll(pid, dll_path);
}
