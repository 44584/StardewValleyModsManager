use crate::total_manager::Manager;
use eframe::egui;

pub struct StardewModsManagerApp {
    manager: Manager,
    selected_profile: Option<String>,
    selected_mods: std::collections::HashSet<String>, // 存储UniqueId
    // Profile创建输入
    new_profile_name: String,
    new_profile_desc: String,
    scanned: bool,
}

impl StardewModsManagerApp {
    pub fn new() -> Self {
        let manager = Manager::default();
        manager.register_all_mods(); // 启动时扫描并注册模组
        Self {
            manager,
            selected_profile: None,
            selected_mods: Default::default(),
            new_profile_name: String::new(),
            new_profile_desc: String::new(),
            scanned: true,
        }
    }

    /// 添加中文字体到 egui
    pub fn add_chinese_font(ctx: &egui::Context) {
        use egui::{FontDefinitions, FontFamily, FontId, TextStyle};

        let mut fonts = FontDefinitions::default();

        // 加载自定义中文字体
        use std::sync::Arc;
        fonts.font_data.insert(
            "my_chinese_font".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansSC-Regular.ttf"
            ))),
        );

        // 将自定义字体添加到比例字体和等宽字体的字体族中
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "my_chinese_font".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("my_chinese_font".to_owned());

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for StardewModsManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::FontFamily;
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(20.0, FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(18.0, FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(16.0, FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(18.0, FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(14.0, FontFamily::Proportional),
            ),
        ]
        .into();
        ctx.set_style(style);

        // 默认选中第一个profile
        if self.selected_profile.is_none() {
            let profiles = self.manager.get_all_profiles();
            if let Some(first) = profiles.first() {
                self.selected_profile = Some(first.name.clone());
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Stardew Mods Manager");
            ui.separator();
            // 查看已注册的模组并多选
            ui.label("ALL MODS");
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    for modinfo in self.manager.get_registered_mods() {
                        let unique_id = &modinfo.manifest_info.UniqueId;
                        let mut checked = self.selected_mods.contains(unique_id);
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut checked, "").changed() {
                                if checked {
                                    self.selected_mods.insert(unique_id.clone());
                                } else {
                                    self.selected_mods.remove(unique_id);
                                }
                            }
                            ui.label(&modinfo.manifest_info.Name);
                            ui.label(&modinfo.manifest_info.Version);
                            ui.label(&modinfo.manifest_info.Description);
                        });
                    }
                });
            if let Some(profile_name) = &self.selected_profile {
                if ui.button("ADD SELECTED MODS TO CURRENT PROFILE").clicked() {
                    let all_mods = self.manager.get_registered_mods();
                    let to_add: Vec<_> = all_mods
                        .into_iter()
                        .filter(|m| self.selected_mods.contains(&m.manifest_info.UniqueId))
                        .collect();
                    self.manager.insert_mods_to_profile(to_add, profile_name);
                }
            }
            ui.separator();
            // profile 增删查
            ui.label("PROFILES");
            let profiles = self.manager.get_all_profiles();
            for profile in &profiles {
                ui.horizontal(|ui| {
                    let selected = self.selected_profile.as_deref() == Some(&profile.name);
                    if ui.selectable_label(selected, &profile.name).clicked() {
                        self.selected_profile = Some(profile.name.clone());
                    }
                    if ui.button("delete").clicked() {
                        self.manager.remove_profile(&profile.name);
                    }
                    ui.label(format!("description: {}", profile.description));
                });
            }
            ui.horizontal(|ui| {
                ui.label("name:");
                ui.text_edit_singleline(&mut self.new_profile_name);
                ui.label("description:");
                ui.text_edit_singleline(&mut self.new_profile_desc);
                if ui.button("new").clicked() {
                    if !self.new_profile_name.trim().is_empty() {
                        self.manager
                            .create_empty_profile(&self.new_profile_name, &self.new_profile_desc);
                        self.new_profile_name.clear();
                        self.new_profile_desc.clear();
                    }
                }
            });
            ui.separator();
            // profile下模组管理
            if let Some(profile_name) = &self.selected_profile {
                ui.label(format!("MODS IN {}", profile_name));
                let mods = self.manager.get_mods_from_profile(profile_name);
                for modinfo in &mods {
                    ui.horizontal(|ui| {
                        ui.label(&modinfo.manifest_info.Name);
                        if ui.button("REMOVE").clicked() {
                            self.manager
                                .remove_mod_from_profile(modinfo.clone(), profile_name);
                        }
                    });
                }
            }
            ui.separator();
            // 选择profile启动游戏
            if let Some(profile_name) = &self.selected_profile {
                if ui.button("LAUNCH").clicked() {
                    self.manager.launch_stardew_valley(profile_name);
                }
            }
        });
    }
}
