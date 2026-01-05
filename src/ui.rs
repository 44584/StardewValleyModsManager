use crate::total_manager::Manager;
use eframe::egui;

pub struct StardewModsManagerApp {
    manager: Manager,
    selected_profile: Option<String>,
}

impl StardewModsManagerApp {
    pub fn new() -> Self {
        Self {
            manager: Manager::default(),
            selected_profile: None,
        }
    }
}

impl eframe::App for StardewModsManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Stardew Mods Manager");
            ui.separator();
            // 1. 查看已注册的模组
            ui.label("all mods:");
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for modinfo in self.manager.get_registered_mods() {
                        ui.horizontal(|ui| {
                            ui.label(&modinfo.manifest_info.Name);
                            ui.label(&modinfo.manifest_info.Version);
                            ui.label(&modinfo.manifest_info.Description);
                        });
                    }
                });
            ui.separator();
            // 2. profile 增删查
            ui.label("Profiles:");
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
                });
            }
            if ui.button("new profile").clicked() {
                self.manager.create_empty_profile("新建配置", "描述");
            }
            ui.separator();
            // 3. profile下模组管理
            if let Some(profile_name) = &self.selected_profile {
                ui.label(format!("mods of {} ", profile_name));
                let mods = self.manager.get_mods_from_profile(profile_name);
                for modinfo in &mods {
                    ui.horizontal(|ui| {
                        ui.label(&modinfo.manifest_info.Name);
                        if ui.button("remove").clicked() {
                            self.manager
                                .remove_mod_from_profile(modinfo.clone(), profile_name);
                        }
                    });
                }
                // 增加模组到profile
                if ui.button("add all other mods").clicked() {
                    let all_mods = self.manager.get_registered_mods();
                    let current_mods = self.manager.get_mods_from_profile(profile_name);
                    let current_ids: std::collections::HashSet<_> = current_mods
                        .iter()
                        .map(|m| &m.manifest_info.UniqueId)
                        .collect();
                    let to_add: Vec<_> = all_mods
                        .into_iter()
                        .filter(|m| !current_ids.contains(&m.manifest_info.UniqueId))
                        .collect();
                    self.manager.insert_mods_to_profile(to_add, profile_name);
                }
            }
            ui.separator();
            // 4. 选择profile启动游戏
            if let Some(profile_name) = &self.selected_profile {
                if ui.button("launch game").clicked() {
                    self.manager.launch_stardew_valley(profile_name);
                }
            }
        });
    }
}
