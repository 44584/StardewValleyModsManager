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
            //设置全局文本样式
            cc.egui_ctx.style_mut(|style| {
                style.text_styles = [
                    (
                        egui::TextStyle::Heading,
                        egui::FontId::new(20.0, egui::FontFamily::Monospace),
                    ),
                    (
                        egui::TextStyle::Body,
                        egui::FontId::new(18.0, egui::FontFamily::Monospace),
                    ),
                    (
                        egui::TextStyle::Button,
                        egui::FontId::new(18.0, egui::FontFamily::Monospace),
                    ),
                ]
                .into();
            });

            Ok(Box::new(StardewModsManagerApp::new(cc)))
        }),
    )
}
