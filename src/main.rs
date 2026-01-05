use eframe::egui;
use StardewModsManager::ui::StardewModsManagerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Stardew Mods Manager",
        options,
        Box::new(|_cc| Ok(Box::new(StardewModsManagerApp::new()))),
    )
}
