#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{self, CreationContext, egui};
use egui::{TextEdit, text_edit::TextEditOutput};
use egui_code_editor::{self, CodeEditor, ColorTheme, Completer, Syntax};

use waql_tool::{waql_query, waql_syntax};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true)
            .with_transparent(false)
            .with_resizable(true)
            .with_maximized(false)
            .with_drag_and_drop(true)
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([280.0, 280.0]),

        ..Default::default()
    };

    eframe::run_native(
        "Waql Tool",
        options,
        Box::new(|cc| Ok(Box::new(WaqlApp::new(cc)))),
    )
}

#[derive(Default)]
struct WaqlApp {
    code: String,
    result: String,
    theme: ColorTheme,
    syntax: Syntax,
    completer: Completer,
    shift: isize,
    numlines_only_natural: bool,
}

impl WaqlApp {
    fn new(_cc: &CreationContext) -> Self {
        let syntax = waql_syntax();
        // let syntax = waql_from_rust();
        WaqlApp {
            code: String::new(),
            result: String::new(),
            theme: ColorTheme::GRUVBOX,
            syntax: syntax.clone(),
            completer: Completer::new_with_syntax(&syntax).with_user_words(),
            shift: 0,
            numlines_only_natural: false,
        }
    }
}

impl eframe::App for WaqlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut editor = CodeEditor::default()
                .id_source("code editor")
                .with_rows(1)
                .with_fontsize(14.0)
                .with_theme(self.theme)
                .with_syntax(self.syntax.to_owned())
                .with_numlines(false)
                .with_numlines_shift(self.shift)
                .with_numlines_only_natural(self.numlines_only_natural)
                .vscroll(false);

            editor.show_with_completer(ui, &mut self.code, &mut self.completer);

            // 检测回车事件
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let query = self.code.clone().trim().to_string();
                match waql_query(&query, None) {
                    Ok(result) => {
                        self.result = format!("{:#?}", result);
                    }
                    Err(e) => {
                        self.result = format!("Error: {}", e);
                    }
                }
            }

            ui.separator();
            ui.horizontal(|h| {
                h.label("Auto-complete TextEdit::singleLine");
                self.completer.show_on_text_widget(
                    h,
                    &self.syntax,
                    &self.theme,
                    |ui| {
                        TextEdit::singleline(&mut self.code)
                            .lock_focus(true)
                            .show(ui)
                    },
                );
            });

            ui.separator();

            if ui.button("Run").clicked() {
                let query = self.code.clone().trim().to_string();
                match waql_query(&query, None) {
                    Ok(result) => {
                        self.result = format!("{:#?}", result);
                    }
                    Err(e) => {
                        self.result = format!("Error: {}", e);
                    }
                }
            }

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.label(&self.result);
                });
        });
    }
}
