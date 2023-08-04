use super::utils;

use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use windows_sys::Win32::{
  Foundation::{CloseHandle, GetLastError, BOOL},
  Globalization,
  System::{
    Diagnostics::{
      Debug::WriteProcessMemory,
      ToolHelp::{CreateToolhelp32Snapshot, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE},
    },
    LibraryLoader::{GetModuleHandleA, GetProcAddress},
    Memory::{VirtualAllocEx, MEM_COMMIT, PAGE_READWRITE},
    Threading::{CreateRemoteThread, PROCESS_ALL_ACCESS},
  },
};

const TRUE: BOOL = 1i32;
const FALSE: BOOL = 0i32;
type HMODULE = isize;

/// 如果 dll 已经存在于 process 中，返回其 hmodule
pub fn check_dll_in_process(pid: u32, dll_name: &str) -> HMODULE {
  let mut dll_name = dll_name.to_string();
  dll_name.push('\0');

  unsafe {
    let h = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);
    let mut module_entry: MODULEENTRY32W = std::mem::zeroed();
    module_entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

    let name_wide: Vec<u16> = dll_name.encode_utf16().collect();
    while Module32NextW(h, &mut module_entry) == TRUE {
      if Globalization::lstrcmpW(name_wide.as_ptr(), module_entry.szModule.as_ptr()) == 0 {
        return module_entry.hModule;
      }
    }

    0
  }
}

pub fn inject_dll<T: Into<PathBuf>>(pid: u32, dll_path: T) {
  let dll_path: PathBuf = dll_path.into();
  let dll_name = dll_path.file_name().unwrap().to_str().unwrap();
  if check_dll_in_process(pid, dll_name) > 0 {
    log::error!("{} 已经注入", dll_name);
    return;
  }

  let dll_path = dll_path.to_str().unwrap();
  unsafe {
    // 打开进程
    let h = utils::open_process(pid, PROCESS_ALL_ACCESS).unwrap();

    //在进程内部申请内存
    let dll_addr = VirtualAllocEx(h, null(), dll_path.len() + 1, MEM_COMMIT, PAGE_READWRITE);
    if dll_addr.is_null() {
      log::error!("内存分配失败");
      CloseHandle(h);
      return;
    }

    //第三步 写入 dll 路径
    if WriteProcessMemory(h, dll_addr, dll_path.as_ptr() as *const c_void, dll_path.len() + 1, null_mut()) == FALSE {
      log::error!("路径写入失败");
      CloseHandle(h);
      return;
    }

    // 执行远程加载函数 loadLibary 加载 dll
    // 获取LoadLibraryA的地址
    // let load_library = GetProcAddress(m_k32, l_name.as_ptr()).unwrap();
    let load_library = get_proc_addr("Kernel32.dll\0", "LoadLibraryA\0").unwrap();

    //远程调用LoadLibrary
    let load_library = Some(*(&load_library as *const _ as *const unsafe extern "system" fn(*mut c_void) -> u32));
    let h_thread = CreateRemoteThread(h, null(), 0, load_library, dll_addr, 0, null_mut());
    if h_thread == 0 {
      log::error!("远程注入失败，error：{}", GetLastError());
    } else {
      log::info!("成功注入：{}", dll_name);
      log::info!("内存地址：{:?}", dll_addr);
    }
    CloseHandle(h_thread);
    CloseHandle(h);
    return;
  }
}

pub fn eject_dll<T: Into<PathBuf>>(pid: u32, dll_path: T) {
  let dll_path: PathBuf = dll_path.into();
  let dll_name = dll_path.file_name().unwrap().to_str().unwrap();

  let h_dll: HMODULE = check_dll_in_process(pid, dll_name);
  if h_dll == 0 {
    log::error!("{} 还没有注入", dll_name);
    return;
  } else {
    log::info!("{}(h_dll: {:0x}) 在进程中，开始卸载...", dll_name, h_dll);
  }

  unsafe {
    // 打开进程
    let h = utils::open_process(pid, PROCESS_ALL_ACCESS).unwrap();

    // FreeLibrary FreeLibraryAndExitThread
    // let free_library = get_proc_addr("FreeLibrary\0").unwrap();
    match get_proc_addr("Kernel32.dll\0", "FreeLibrary\0") {
      Ok(free_library) => {
        //远程调用 FreeLibrary
        let free_library = Some(*(&free_library as *const _ as *const unsafe extern "system" fn(*mut c_void) -> u32));
        let h_thread = CreateRemoteThread(h, null(), 0, free_library, h_dll as *const isize as *const c_void, 0, null_mut());
        if h_thread == 0 || GetLastError() != 0 {
          log::error!("远程卸载失败，error：{}", GetLastError());
        } else {
          log::info!("成功卸载：{}", dll_name);
        }
        CloseHandle(h_thread);
      }
      Err(e) => {
        log::error!("{}", e);
      }
    }
    CloseHandle(h);
  }
}

/// proc_name: 进程名，需要以 '\0' 结尾
fn get_proc_addr(module_name: &str, proc_name: &str) -> Result<unsafe extern "system" fn() -> isize, Box<dyn std::error::Error>> {
  unsafe {
    // 执行远程加载函数 FreeLibary 加载 dll
    // let k_name: Vec<u16> = String::from("Kernel32.dll\0").encode_utf16().collect();
    let k_name = module_name;
    let m_k32 = GetModuleHandleA(k_name.as_ptr()); //获取Kernel32的基地址
    if m_k32 == 0 {
      Err(format!("获取Kernel32的基地址失败, m_k32: {:0x}, err: {}", m_k32, GetLastError()))?
    }
    log::trace!("Kernel32 基地址: {:0x}", m_k32);

    //获取LoadLibraryA的地址
    let proc_addr = GetProcAddress(m_k32, proc_name.as_ptr()).unwrap();
    log::trace!("proc_name: {}, addr: {:?}", proc_name, proc_addr);
    Ok(proc_addr)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use log::LevelFilter;
  use simple_logger::SimpleLogger;

  #[test]
  fn test_get_proc_addr() {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();
    // get_proc_addr("LoadLibraryA\0").unwrap();
    get_proc_addr("Kernel32.dll\0", "FreeLibrary\0").unwrap();
  }
}
