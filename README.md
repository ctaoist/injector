向 Windows 程序注入 dll。

## Usage

```
Options:
      --debug        Turn debugging information on
  -i                 inject the dll
  -e                 eject the dll
  -d <DLL_PATH>      dll path or name
  -h, --help         Print help
  -V, --version      Print version
```

```sh
## 注入
injector WeChat.exe -i -d D:\dllTest.dll

## 卸载
injector WeChat.exe -e dllTest.dll
```

## 作为库引用

`Cargo.toml`:

```
[dependencies]
injector = {git = "https://github.com/ctaoist/injector"}
```

```rust
use injector::{get_process_id, inject_dll, eject_dll};

let pid = get_process_id("WeChat.exe\0");
inject_dll(pid, "C:\dllTest.dll");
```

## Build 

```
git clone https://github.com/ctaoist/injector

cd injector
cargo build --bin injector --features="build-binary"
```

如果使用 `Xargo` 编译，需要使用 nightly rustc。