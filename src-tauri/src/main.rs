#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("wojak-world is a Windows-only app.");
        std::process::exit(1);
    }
    #[cfg(windows)]
    wojak_world_lib::run();
}
