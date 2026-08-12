use eframe::egui;
use pacord::profile::UserProfile;
use std::collections::HashMap;

struct PacordApp {
    profile: UserProfile,
    is_host: bool,
    zerotier_ip: String,
    port: u16,
    connected_clients: Vec<ClientInfo>,
    permissions: HashMap<usize, ClientPermissions>,
    customization_open: bool,
}

#[derive(Clone)]
struct ClientInfo {
    id: usize,
    nickname: String,
    ip: String,
    has_controller: bool,
    cursor_pos: egui::Pos2,
}

#[derive(Clone, Copy)]
struct ClientPermissions {
    can_mouse: bool,
    can_keyboard: bool,
    can_controller: bool,
}

impl Default for PacordApp {
    fn default() -> Self {
        let profile = UserProfile::load_from_file("pacord_profile.toml").unwrap_or_default();
        let mut permissions = HashMap::new();
        // Pre-populate mock slots up to 8 users
        for i in 1..=8 {
            permissions.insert(
                i,
                ClientPermissions {
                    can_mouse: true,
                    can_keyboard: true,
                    can_controller: false,
                },
            );
        }

        Self {
            profile,
            is_host: true,
            zerotier_ip: "10.147.20.5".to_string(),
            port: 7777,
            connected_clients: vec![
                ClientInfo {
                    id: 1,
                    nickname: "Alice".to_string(),
                    ip: "10.147.20.12".to_string(),
                    has_controller: true,
                    cursor_pos: egui::pos2(200.0, 150.0),
                },
                ClientInfo {
                    id: 2,
                    nickname: "Bob".to_string(),
                    ip: "10.147.20.15".to_string(),
                    has_controller: false,
                    cursor_pos: egui::pos2(450.0, 300.0),
                },
            ],
            permissions,
            customization_open: false,
        }
    }
}

impl eframe::App for PacordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Enforce strict Black and White color scheme
        let mut visuals = egui::Visuals::dark();
        visuals.dark_mode = true;
        visuals.window_fill = egui::Color32::BLACK;
        visuals.panel_fill = egui::Color32::BLACK;
        visuals.override_text_color = Some(egui::Color32::WHITE);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::BLACK;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        visuals.widgets.inactive.bg_fill = egui::Color32::BLACK;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(40);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::WHITE);
        visuals.widgets.active.bg_fill = egui::Color32::from_gray(80);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, egui::Color32::WHITE);
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PACORD — Remote Collaboration (Black & White)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Customization").clicked() {
                        self.customization_open = !self.customization_open;
                    }
                    if ui
                        .button(if self.is_host {
                            "Mode: Host"
                        } else {
                            "Mode: Client"
                        })
                        .clicked()
                    {
                        self.is_host = !self.is_host;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.customization_open {
                ui.heading("Device Customization Profile");
                ui.separator();
                ui.label(format!("Nickname: {}", self.profile.nickname));
                ui.add(
                    egui::Slider::new(&mut self.profile.mouse.sensitivity, 0.1..=5.0)
                        .text("Mouse Sensitivity"),
                );
                ui.add(
                    egui::Slider::new(&mut self.profile.controller.deadzone, 0.01..=0.5)
                        .text("Controller Deadzone"),
                );
                ui.checkbox(
                    &mut self.profile.controller.invert_y,
                    "Invert Controller Y Axis",
                );
                if ui.button("Save Profile").clicked() {
                    let _ = self.profile.save_to_file("pacord_profile.toml");
                }
                if ui.button("Close Customization").clicked() {
                    self.customization_open = false;
                }
                return;
            }

            ui.columns(2, |columns| {
                columns[0].heading("Session & Network (ZeroTier)");
                columns[0].add(
                    egui::TextEdit::singleline(&mut self.zerotier_ip).hint_text("ZeroTier IP"),
                );
                columns[0].add(egui::DragValue::new(&mut self.port).prefix("Port: "));

                if self.is_host {
                    if columns[0].button("Start Hosting (Max 8 Users)").clicked() {
                        // Host initialization logic
                    }
                    columns[0].separator();
                    columns[0].label("Connected Clients (Up to 8):");

                    let mut to_remove = None;
                    for client in &self.connected_clients {
                        columns[0].horizontal(|ui| {
                            ui.label(format!(
                                "[{}] {} ({})",
                                client.id, client.nickname, client.ip
                            ));
                            if client.has_controller {
                                ui.label(" [GAMEPAD]");
                            }
                            if let Some(perms) = self.permissions.get_mut(&client.id) {
                                ui.checkbox(&mut perms.can_mouse, "Mouse");
                                ui.checkbox(&mut perms.can_keyboard, "Key");
                                ui.checkbox(&mut perms.can_controller, "Ctrl");
                            }
                            if ui.button("Disconnect").clicked() {
                                to_remove = Some(client.id);
                            }
                        });
                    }
                    if let Some(id) = to_remove {
                        self.connected_clients.retain(|c| c.id != id);
                    }
                } else {
                    if columns[0].button("Connect to Host").clicked() {
                        // Client connection logic
                    }
                    columns[0].separator();
                    columns[0].label("Options:");
                    if columns[0]
                        .button("Create Isolated Session (Cage/Xephyr)")
                        .clicked()
                    {
                        // Isolated session logic
                    }
                }

                columns[1].heading("Host Screen Simulation / Overlay View");
                columns[1].separator();

                // Interactive canvas simulating host desktop with cursors and controller badges
                let (response, painter) =
                    columns[1].allocate_painter(egui::vec2(400.0, 300.0), egui::Sense::hover());
                let rect = response.rect;

                // Draw background box
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(15));
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, egui::Color32::WHITE));

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Host Desktop Stream\n(PipeWire / XShm)",
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );

                // Render overlays for each connected client: cursor square with nickname & gamepad icon
                for client in &self.connected_clients {
                    let screen_pos = rect.min + client.cursor_pos.to_vec2();
                    if rect.contains(screen_pos) {
                        // Draw cursor square
                        let cursor_rect =
                            egui::Rect::from_min_size(screen_pos, egui::vec2(12.0, 12.0));
                        painter.rect_filled(cursor_rect, 0.0, egui::Color32::WHITE);

                        // Draw nickname badge next to cursor
                        painter.text(
                            screen_pos + egui::vec2(16.0, 0.0),
                            egui::Align2::LEFT_TOP,
                            &client.nickname,
                            egui::FontId::proportional(12.0),
                            egui::Color32::WHITE,
                        );

                        // Draw controller icon at the top if active
                        if client.has_controller {
                            let icon_rect = egui::Rect::from_min_size(
                                screen_pos + egui::vec2(0.0, -20.0),
                                egui::vec2(16.0, 10.0),
                            );
                            painter.rect_stroke(
                                icon_rect,
                                2.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                            );
                            painter.text(
                                screen_pos + egui::vec2(18.0, -22.0),
                                egui::Align2::LEFT_TOP,
                                "[PAD]",
                                egui::FontId::proportional(10.0),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "PACORD — Remote Collaboration",
        options,
        Box::new(|_cc| Ok(Box::new(PacordApp::default()))),
    )
}
