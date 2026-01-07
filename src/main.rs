use StardewModsManager::ui::StardewModsManagerApp;
use eframe::egui;
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(false) // 启动时全屏
            .with_maximized(true), // 确保不是最大化而是真正的全屏
        ..Default::default()
    };
    eframe::run_native(
        "Stardew Mods Manager",
        options,
        Box::new(|cc| {
            // 在创建应用之前设置字体
            StardewModsManagerApp::add_chinese_font(&cc.egui_ctx);
            Ok(Box::new(StardewModsManagerApp::new(cc)))
        }),
    )
}
