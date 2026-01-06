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
    // 支持自定义模组文件夹目录和smapi路径
    mods_folder_input: String,
    smapi_path_input: String,
    is_beginner: bool,
}

impl StardewModsManagerApp {
    pub fn new() -> Self {
        let manager = Manager::default();

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("StardewModsManager");
        let config_path = data_dir.join("setting.toml");
        let is_beginner = !config_path.exists();
        Self {
            manager,
            selected_profile: None,
            selected_mods: Default::default(),
            new_profile_name: String::new(),
            new_profile_desc: String::new(),
            scanned: true,
            mods_folder_input: String::new(),
            smapi_path_input: String::new(),
            is_beginner,
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
            if self.is_beginner {
                ui.heading("首次使用? 请填写模组文件夹路径和smapi路径");
                self.mods_folder_input = String::from(
                    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley\\Mods",
                );
                self.smapi_path_input = String::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley/StardewModdingAPI.exe");
                ui.horizontal(|ui| {
                    ui.vertical(|ui|{
                        ui.horizontal(|ui| {
                            ui.label("Mods 文件夹路径:");
                            ui.add(egui::TextEdit::singleline(&mut self.mods_folder_input).desired_width(1000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("SMAPI路径:");
                            ui.add(egui::TextEdit::singleline(&mut self.smapi_path_input).desired_width(1000.0));
                        });

                        if ui.button("保存").clicked() {
                        if !self.mods_folder_input.trim().is_empty() {
                            let data_dir = dirs::data_dir()
                                .unwrap_or_else(|| std::env::current_dir().unwrap())
                                .join("StardewModsManager");
                            let config_path = data_dir.join("setting.toml");
                            let cfg = crate::config::AppConfig {
                                mods_folder_path: self.mods_folder_input.clone(),
                                smapi_path: self.smapi_path_input.clone(),
                            };
                            if let Err(e) = cfg.save_to_file(&config_path) {
                                ui.label(format!("保存失败: {}", e));
                            } else {
                                // 设置scanner路径并隐藏设置界面
                                self.manager.set_scanner_mods_path(std::path::PathBuf::from(
                                    &self.mods_folder_input,
                                ));
                                self.is_beginner = false;
                                self.manager.register_all_mods();
                            }
                        }
                    }
                    }
                    );
                   
                    
                });
                ui.separator();
            }
            ui.separator();
            // 查看已注册的模组并多选
            ui.label("所有模组");
            // 只有填写并保存路径后才显示扫描按钮
            if !self.is_beginner {
                if ui.button("扫描模组").clicked() {
                    self.manager.register_all_mods();
                    self.selected_mods.clear();
                }
            }
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
                let button_content = format!(
                    "选中的模组添加到{}",
                    self.selected_profile.as_ref().unwrap()
                );
                if ui.button(button_content).clicked() {
                    let all_mods = self.manager.get_registered_mods();
                    let to_add: Vec<_> = all_mods
                        .into_iter()
                        .filter(|m| self.selected_mods.contains(&m.manifest_info.UniqueId))
                        .collect();
                    self.manager.insert_mods_to_profile(to_add, profile_name);
                    // 然后清空选中的模组
                    self.selected_mods.clear();
                }
            }
            ui.separator();
            // profile 增删查
            ui.label("所有配置");
            let profiles = self.manager.get_all_profiles();
            for profile in &profiles {
                ui.horizontal(|ui| {
                    let selected = self.selected_profile.as_deref() == Some(&profile.name);
                    if ui.selectable_label(selected, &profile.name).clicked() {
                        self.selected_profile = Some(profile.name.clone());
                    }
                    if ui.button("删除配置").clicked() {
                        match self.manager.remove_profile(&profile.name) {
                            Ok(n) => {
                                if n == 0 {
                                    self.selected_mods.clear();
                                }
                            }
                            Err(e) => {
                                ui.horizontal(|ui| {
                                    ui.label(format!("删除失败:{}", e));
                                });
                            }
                        }
                    }
                    ui.label(format!("信息: {}", profile.description));
                });
            }
            ui.horizontal(|ui| {
                ui.label("配置名:");
                ui.text_edit_singleline(&mut self.new_profile_name);
                ui.label("配置信息:");
                ui.text_edit_singleline(&mut self.new_profile_desc);
                if ui.button("创建").clicked() {
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
                ui.label(format!("{}的模组", profile_name));
                let mods = self.manager.get_mods_from_profile(profile_name);
                for modinfo in &mods {
                    ui.horizontal(|ui| {
                        ui.label(&modinfo.manifest_info.Name);
                        if ui.button("从配置中移除").clicked() {
                            self.manager
                                .remove_mod_from_profile(modinfo.clone(), profile_name);
                        }
                    });
                }
            }
            ui.separator();
            // 选择profile启动游戏
            if let Some(profile_name) = &self.selected_profile {
                if ui.button("启动").clicked() {
                    self.manager.launch_stardew_valley(profile_name);
                }
            }
        });
    }
}
