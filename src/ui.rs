use crate::mods_manager::ManifestInfo;
use crate::mods_manager::ModInfo;
use crate::total_manager::Manager;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

pub struct StardewModsManagerApp {
    manager: Manager,
    selected_profile: Option<String>,
    selected_mods: std::collections::HashSet<String>, // 存储UniqueId
    // Profile创建输入
    new_profile_name: String,
    new_profile_desc: String,
    // 支持自定义模组文件夹目录和smapi路径
    data_dir: PathBuf,
    mods_folder_input: String,
    smapi_path_input: String,
    is_beginner: bool,
    // 确认对话框状态
    show_reset_confirmation: bool,
}

impl StardewModsManagerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
            data_dir,
            new_profile_name: String::new(),
            new_profile_desc: String::new(),
            mods_folder_input: String::new(),
            smapi_path_input: String::new(),
            is_beginner,
            show_reset_confirmation: false,
        }
    }

    /// 添加中文字体到 egui
    pub fn add_chinese_font(ctx: &egui::Context) {
        use egui::{FontDefinitions, FontFamily};

        let mut fonts = FontDefinitions::default();

        // 优先尝试使用系统字体
        #[cfg(target_os = "windows")]
        let system_fonts = vec!["Microsoft YaHei", "SimHei", "SimSun"];
        #[cfg(target_os = "macos")]
        let system_fonts = vec!["PingFang SC", "Heiti SC", "STHeiti"];
        #[cfg(target_os = "linux")]
        let system_fonts = vec!["WenQuanYi Micro Hei", "Noto Sans CJK SC"];

        // 检查系统字体是否可用
        let mut found_system_font = false;
        for font_name in system_fonts {
            if fonts
                .families
                .get(&FontFamily::Proportional)
                .map_or(false, |fonts| fonts.contains(&font_name.to_string()))
            {
                found_system_font = true;
                break;
            }
        }

        // 只有在系统字体不可用时才加载自定义字体
        if !found_system_font {
            fonts.font_data.insert(
                "my_chinese_font".to_owned(),
                Arc::new(egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/NotoSansSC-Regular.ttf"
                ))),
            );

            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "my_chinese_font".to_owned());
        }

        ctx.set_fonts(fonts);
    }

    fn ui_beginner_setting(&mut self, ui: &mut egui::Ui) {
        ui.heading("首次使用?请填写模组文件夹路径和smapi路径, 格式默认如下:");
        ui.label("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley\\Mods");
        ui.label("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley\\StardewModdingAPI.exe");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Mods 文件夹路径: ");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.mods_folder_input)
                            .desired_width(1100.0)
                            .background_color(egui::Color32::from_rgb(230, 240, 255))
                            .frame(true),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("StardewModdingAPI.exe 路径:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.smapi_path_input)
                            .desired_width(1000.0)
                            .background_color(egui::Color32::from_rgb(230, 240, 255))
                            .frame(true),
                    );
                });

                if ui.button("保存").highlight().clicked() {
                    if (!self.mods_folder_input.trim().is_empty())
                        && (!self.smapi_path_input.trim().is_empty())
                        && (self.smapi_path_input.trim().ends_with("exe"))
                    {
                        let config_path = self.data_dir.join("setting.toml");
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
            });
        });
    }

    /// - mods列表 组件
    /// - 提供选中功能
    /// Todo: 删除操作后续改为先收集删除名单, 再统一删除
    fn ui_mods_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("所有模组");
        // 只有填写并保存路径后才显示扫描按钮
        if !self.is_beginner {
            if ui.button("扫描模组").highlight().clicked() {
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
            if ui.button(button_content).highlight().clicked() {
                let all_mods = self.manager.get_registered_mods();
                let to_add: Vec<_> = all_mods
                    .iter()
                    .filter(|m| self.selected_mods.contains(&m.manifest_info.UniqueId))
                    .cloned()
                    .collect();
                self.manager.insert_mods_to_profile(to_add, profile_name);
                // 然后清空选中的模组
                self.selected_mods.clear();
            }
        }
    }

    /// profile列表 组件
    /// Todo: 删除操作后续改为先收集删除名单, 再统一删除
    fn ui_profile_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("所有配置");

        let mut profiles_to_delete = Vec::new();
        for profile in self.manager.get_all_profiles() {
            ui.horizontal(|ui| {
                let selected = self.selected_profile.as_deref() == Some(&profile.name);
                if ui
                    .selectable_label(selected, &profile.name)
                    .highlight()
                    .clicked()
                {
                    self.selected_profile = Some(profile.name.clone());
                }
                if ui.button("删除配置").highlight().clicked() {
                    profiles_to_delete.push(profile.name.clone());
                }
                ui.label(format!("信息: {}", profile.description));
            });
        }

        for profile_name in profiles_to_delete {
            match self.manager.remove_profile(&profile_name) {
                Ok(n) => {
                    if n == 0 {
                        self.selected_profile = None;
                        self.selected_mods.clear();
                    }
                }
                Err(e) => {
                    eprintln!("删除失败: {}", e);
                }
            }
        }
    }

    /// 创建profile的ui
    fn ui_new_profile(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("配置名:");
            ui.text_edit_singleline(&mut self.new_profile_name)
                .highlight();
            ui.label("配置信息:");
            ui.text_edit_singleline(&mut self.new_profile_desc)
                .highlight();
            if ui.button("创建").highlight().clicked() {
                if !self.new_profile_name.trim().is_empty() {
                    self.manager
                        .create_empty_profile(&self.new_profile_name, &self.new_profile_desc);
                    self.new_profile_name.clear();
                    self.new_profile_desc.clear();
                }
            }
        });
    }

    /// - 显示选中的配置下的模组
    /// - 提供移除按钮
    fn ui_mods_in_profile(&mut self, ui: &mut egui::Ui) {
        if let Some(profile_name) = &self.selected_profile {
            ui.label(format!("{}的模组", profile_name));
            let mods = self.manager.get_mods_from_profile(profile_name);
            for modinfo in &mods {
                ui.horizontal(|ui| {
                    ui.label(&modinfo.manifest_info.Name);
                    if ui.button("从配置中移除").highlight().clicked() {
                        self.manager
                            .remove_mod_from_profile(modinfo.clone(), profile_name);
                    }
                });
            }
        }
    }
}

impl eframe::App for StardewModsManagerApp {
    /// Todo: ui实现组件化
    /// Todo: 模组总览, profile管理 作为两个页面展示(通过按钮调整), 1)减轻静止时内存占用; 2)更清晰的展示
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 默认选中第一个profile
        if self.selected_profile.is_none() {
            let profiles = self.manager.get_all_profiles();
            if let Some(first) = profiles.first() {
                self.selected_profile = Some(first.name.clone());
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_beginner {
                self.ui_beginner_setting(ui);
            } else {
                //不是beginner可以通过重置成为beginner, 然后修改配置
                if ui.button("RESET").highlight().clicked() {
                    self.show_reset_confirmation = true;
                }
            }
            // 显示确认对话框
            if self.show_reset_confirmation {
                egui::Window::new("确认重置")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(
                            "警告:此操作将清空的SMAPI路径和m
                        Mods路径!",
                        );
                        ui.label("确定要继续吗? ");

                        ui.horizontal(|ui| {
                            if ui.button("取消").highlight().clicked() {
                                self.show_reset_confirmation = false;
                            }
                            if ui.button("确认重置").highlight().clicked() {
                                // 执行真正的重置操作
                                self.manager.reset();
                                self.is_beginner = true;
                                self.show_reset_confirmation = false;
                            }
                        });
                    });
            }
            ui.separator();
            self.ui_mods_list(ui);
            ui.separator();

            self.ui_profile_list(ui);
            self.ui_new_profile(ui);
            ui.separator();

            self.ui_mods_in_profile(ui);
            ui.separator();
            // 选择profile启动游戏
            if let Some(profile_name) = &self.selected_profile {
                if ui.button("启动").highlight().clicked() {
                    self.manager.launch_stardew_valley(profile_name);
                }
            }
        });
    }
}
