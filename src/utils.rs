use windows_sys::Win32::{
  Foundation::{GetLastError, BOOL, HANDLE},
  Globalization::lstrcmpiW,
  System::{
    Diagnostics::{
      Debug::{
        FormatMessageA, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_HMODULE, FORMAT_MESSAGE_FROM_SYSTEM,
        FORMAT_MESSAGE_IGNORE_INSERTS,
      },
      ToolHelp::{CreateToolhelp32Snapshot, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
    },
    Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS},
  },
};

const FALSE: BOOL = 0i32;

/// 目前还不能正确打印出错误信息
#[allow(unused)]
pub fn get_err_msg(eid: u32) -> String {
  unsafe {
    let mut buf = [0_u8; 1024];
    FormatMessageA(
      FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_ALLOCATE_BUFFER,
      std::ptr::null(),
      eid,
      1024,
      &mut buf as *mut u8,
      1024,
      std::ptr::null(),
    );
    let e = GetLastError();
    if e > 0 {
      log::error!("FormatMessage error: {e}");
    }
    String::from_utf8((&buf).to_vec()).unwrap()
  }
}

pub fn open_process(pid: u32, rights: PROCESS_ACCESS_RIGHTS) -> Result<HANDLE, Box<dyn std::error::Error>> {
  unsafe {
    let h = OpenProcess(rights, FALSE, pid);
    if h == 0 || GetLastError() != 0 {
      Err(format!("进程打开失败，可能权限不足或者关闭了应用，error : {}", GetLastError()))?
    }
    Ok(h)
  }
}

pub fn get_process_id(name: &str) -> Result<u32, Box<dyn std::error::Error>> {
  log::info!("开始搜索: {name}");
  let name_wide: Vec<u16> = name.encode_utf16().collect();
  //   name_wide.push('\0');
  unsafe {
    let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    let mut info: PROCESSENTRY32W = std::mem::zeroed();
    info.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    while Process32NextW(handle, &mut info) == 1 {
      if lstrcmpiW(info.szExeFile.as_ptr(), name_wide.as_ptr()) == 0 {
        log::info!("{} pid: {}", name, info.th32ProcessID);
        return Ok(info.th32ProcessID);
      }
    }
  };
  Err(format!("未能找到 {name} 进程!搜索失败!"))?
}

/// 文件/文件夹是否存在
pub fn file_exist<T: Into<std::path::PathBuf>>(fname: T) -> bool {
  let fname: std::path::PathBuf = fname.into();
  fname.exists()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_error_msg() {
    println!("err msg: {}", get_err_msg(18));
  }
}
