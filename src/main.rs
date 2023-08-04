#![no_main]
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod inject;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// Process name to inject or eject
  #[arg(required = true)]
  name: String,

  /// Turn debugging information on
  #[arg(long)]
  debug: bool,

  /// inject the dll
  #[arg(short)]
  inject: bool,

  /// eject the dll
  #[arg(short, conflicts_with = "inject")]
  eject: bool,

  /// dll path or name
  #[arg(short)]
  dll_path: String,
}

#[no_mangle]
pub extern "C" fn main() {
  let cli = Cli::parse();
  if cli.debug {
    SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
  } else {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
  }

  let mut name = cli.name;
  name.push('\0');

  let pid = utils::get_process_id(name.as_str()).unwrap();

  if cli.inject {
    if !utils::file_exist(&cli.dll_path) {
      log::error!("{} 不存在", cli.dll_path);
      return;
    }

    inject::inject_dll(pid, &cli.dll_path);
  }

  if cli.eject {
    inject::eject_dll(pid, &cli.dll_path);
  }
}
