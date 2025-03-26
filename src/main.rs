use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Reflect",
        options,
        Box::new(|cc| {
            // Configure the background color to white and text color to black
            let mut visuals = cc.egui_ctx.style().visuals.clone();
            visuals.window_fill = egui::Color32::WHITE;
            visuals.panel_fill = egui::Color32::WHITE;
            visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.inactive.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.hovered.fg_stroke.color = egui::Color32::BLACK;
            visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;
            cc.egui_ctx.set_visuals(visuals);
            
            Box::new(ReflectApp::default())
        }),
    )
}

#[derive(Default)]
struct ReflectApp {
    day_summary: String,
    selected_emoji: Option<usize>,
    detailed_notes: String,
    current_action: String,
    actions: Vec<String>,
}

impl ReflectApp {
    fn render_app_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Add some padding from the left
            ui.add_space(10.0);
            
            // Create a circle frame for the icon
            let circle_size = 24.0;
            let circle_frame = egui::Frame::none()
                .fill(egui::Color32::from_gray(200))
                .rounding(circle_size / 2.0);
            
            circle_frame.show(ui, |ui| {
                ui.allocate_space(egui::vec2(circle_size, circle_size));
            });
            
            ui.add_space(8.0); // Space between circle and text
            
            // Add the "Reflect" text
            ui.heading("Reflect");
        });
        ui.add_space(10.0); // Space after the header
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        let header_frame = egui::Frame::none()
            .fill(egui::Color32::from_gray(160))  // Lighter gray to match design
            .inner_margin(egui::style::Margin::symmetric(20.0, 40.0))  // More vertical padding
            .outer_margin(egui::style::Margin::symmetric(0.0, 0.0));  // No outer margin
            
        header_frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("Add Your Daily Note")
                        .color(egui::Color32::WHITE)
                        .size(24.0)
                );
                ui.add_space(5.0);  // Space between title and subtitle
                ui.label(
                    egui::RichText::new("Reflect on your day and jot down your thoughts")
                        .color(egui::Color32::WHITE)
                );
            });
        });
        ui.add_space(20.0);
    }

    fn render_emoji_selection(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let emojis = ["😊", "🙂", "😐", "😕", "😢"];
        
        ui.horizontal_centered(|ui| {
            for (idx, emoji) in emojis.iter().enumerate() {
                let button = egui::Button::new(
                    egui::RichText::new(*emoji).size(32.0)
                ).frame(false);
                
                if ui.add(button).clicked() {
                    self.selected_emoji = Some(idx);
                }
                if idx < emojis.len() - 1 {
                    ui.add_space(20.0);
                }
            }
        });
        ui.add_space(10.0);
        
        ui.vertical_centered(|ui| {
            ui.label("Select an emoji that represents your day");
        });
        ui.add_space(20.0);
    }

    fn render_text_box(ui: &mut egui::Ui, title: &str, text: &mut String, multiline: bool) {
        let text_frame = egui::Frame::none()
            .fill(egui::Color32::from_gray(245))
            .rounding(8.0)
            .inner_margin(10.0)
            .outer_margin(10.0);

        text_frame.show(ui, |ui| {
            ui.add_space(5.0);
            ui.heading(title);
            ui.add_space(10.0);
            if multiline {
                let text_edit = egui::TextEdit::multiline(text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(4)
                    .frame(false);
                ui.add(text_edit);
            } else {
                let text_edit = egui::TextEdit::singleline(text)
                    .desired_width(f32::INFINITY)
                    .frame(false);
                ui.add(text_edit);
            }
            ui.add_space(5.0);
        });
    }
}

impl eframe::App for ReflectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // App header with circle icon
            self.render_app_header(ui);
            
            // Header
            self.render_header(ui);
            
            // Content area with no side margins
            let content_frame = egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(0.0, 0.0));
            
            content_frame.show(ui, |ui| {
                // How was your day? input
                Self::render_text_box(ui, "How was your day?", &mut self.day_summary, false);
                
                // Emoji selection
                self.render_emoji_selection(ui);
                
                // Free text box
                Self::render_text_box(ui, "Free Text Box", &mut self.detailed_notes, true);
                ui.add_space(10.0);
                
                // Actions Entry
                ui.heading("Actions Entry");
                ui.add_space(5.0);
                ui.label("Log your actions here");
                
                let action_frame = egui::Frame::none()
                    .fill(egui::Color32::from_gray(245))
                    .rounding(8.0)
                    .inner_margin(10.0)
                    .outer_margin(10.0);

                action_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let text_edit = egui::TextEdit::singleline(&mut self.current_action)
                            .desired_width(ui.available_width() - 100.0)
                            .frame(false)
                            .hint_text("What did you do today?");
                        ui.add(text_edit);
                        
                        if ui.button("Add Action").clicked() && !self.current_action.is_empty() {
                            self.actions.push(self.current_action.clone());
                            self.current_action.clear();
                        }
                    });
                    
                    if !self.actions.is_empty() {
                        ui.add_space(10.0);
                        let mut action_to_remove = None;
                        for (idx, action) in self.actions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}. {}", idx + 1, action));
                                if ui.small_button("🗑").clicked() {
                                    action_to_remove = Some(idx);
                                }
                            });
                        }
                        if let Some(idx) = action_to_remove {
                            self.actions.remove(idx);
                        }
                    }
                });
            });
        });
    }
} 