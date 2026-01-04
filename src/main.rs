mod link_manager;
mod mods_manager;
mod total_manager;
use std::process::Command;

fn main() {
    let smapi_path =
        "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/StardewModdingAPI.exe";
    let mods_folder_path = "Mods_simple";

    launch_stardew_valley(smapi_path, mods_folder_path);
}

fn launch_stardew_valley(smapi_path: &str, mods_folder_path: &str) {
    let child = Command::new(smapi_path)
        .arg("--mods-path")
        .arg(mods_folder_path)
        .spawn()
        .unwrap();
    eprintln!("{}已启动", child.id());
}
