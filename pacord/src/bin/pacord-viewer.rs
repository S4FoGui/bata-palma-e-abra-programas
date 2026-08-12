use eframe::egui;
use pacord::transport::{connect_client, FramePacket};
use std::env;
use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver};
use std::thread;

struct ViewerApp {
    frames: Receiver<FramePacket>,
    texture: Option<egui::TextureHandle>,
    status: String,
    last_sequence: u64,
}

impl ViewerApp {
    fn new(frames: Receiver<FramePacket>) -> Self {
        Self {
            frames,
            texture: None,
            status: "Aguardando frames autorizados…".to_string(),
            last_sequence: 0,
        }
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(packet) = self.frames.try_recv() {
            match image::load_from_memory(&packet.jpeg) {
                Ok(image) => {
                    let rgba = image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                    self.texture = Some(ctx.load_texture(
                        "pacord-remote-screen",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.last_sequence = packet.sequence;
                    self.status = format!(
                        "Recebendo {}×{} — frame {}",
                        packet.width, packet.height, packet.sequence
                    );
                }
                Err(error) => self.status = format!("Frame inválido: {error}"),
            }
        }

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::BLACK;
        visuals.panel_fill = egui::Color32::BLACK;
        visuals.override_text_color = Some(egui::Color32::WHITE);
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PACORD — janela remota");
                ui.separator();
                ui.label(&self.status);
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                let available = ui.available_size();
                let size = texture.size_vec2();
                let scale = (available.x / size.x).min(available.y / size.y).min(1.0);
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), size * scale));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("O host ainda não enviou um frame.");
                });
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn usage() {
    eprintln!(
        "Uso: PACORD_SECRET='<segredo>' cargo run --bin pacord-viewer -- <host-zerotier-ip:porta> <apelido>\n\nExemplo:\n  PACORD_SECRET='troque-por-um-segredo-longo' cargo run --bin pacord-viewer -- 10.147.20.5:7777 Alice"
    );
}

fn main() -> Result<(), eframe::Error> {
    let mut args = env::args().skip(1);
    let Some(server) = args.next() else {
        usage();
        return Ok(());
    };
    let Some(nickname) = args.next() else {
        usage();
        return Ok(());
    };
    let secret = match env::var("PACORD_SECRET") {
        Ok(value) if value.len() >= 16 => value.into_bytes(),
        _ => {
            eprintln!("PACORD_SECRET deve existir e ter pelo menos 16 bytes.");
            return Ok(());
        }
    };
    let server: SocketAddr = match server.parse() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("endereço inválido: {error}");
            return Ok(());
        }
    };

    let (frame_sender, frame_receiver) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime Tokio");
        runtime.block_on(async move {
            match connect_client(server, nickname, secret).await {
                Ok(mut frames) => {
                    while let Some(result) = frames.recv().await {
                        match result {
                            Ok(packet) => {
                                if frame_sender.send(packet).is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                eprintln!("transporte PACORD: {error}");
                                break;
                            }
                        }
                    }
                }
                Err(error) => eprintln!("não foi possível conectar ao PACORD: {error}"),
            }
        });
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "PACORD — janela remota",
        options,
        Box::new(|_cc| Ok(Box::new(ViewerApp::new(frame_receiver)))),
    )
}
