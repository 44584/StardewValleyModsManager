use StardewModsManager::ui::StardewModsManagerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Stardew Mods Manager",
        options,
        Box::new(|cc| {
            // 在创建应用之前设置字体
            StardewModsManagerApp::add_chinese_font(&cc.egui_ctx);
            Ok(Box::new(StardewModsManagerApp::new()))
        }),
    )
}
