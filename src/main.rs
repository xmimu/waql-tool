#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{self, CreationContext, egui};
use egui::{TextBuffer, TextEdit};
use egui_code_editor::{ColorTheme, Completer, Syntax, Token};

use serde_json::Value;
use serde_json::json;
use serde_json::to_string_pretty;
use waapi_rs::WaapiClient;
use waql_tool::WAAPI_ACCESSORS;
use waql_tool::WAAPI_PROPERTIES;
use waql_tool::waql_syntax;

// UI 常量
const APP_TITLE: &str = "Waql Tool";
const DEFAULT_WINDOW_WIDTH: f32 = 900.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;
const MIN_WINDOW_SIZE: f32 = 280.0;
const DEFAULT_FONT_SIZE: f32 = 18.0;
const INPUT_HINT_TEXT: &str = "Enter the WAQL statement here.";

/// 可用的代码编辑器主题列表
const THEMES: [ColorTheme; 8] = [
    ColorTheme::AYU,
    ColorTheme::AYU_MIRAGE,
    ColorTheme::AYU_DARK,
    ColorTheme::GITHUB_DARK,
    ColorTheme::GITHUB_LIGHT,
    ColorTheme::GRUVBOX,
    ColorTheme::GRUVBOX_LIGHT,
    ColorTheme::SONOKAI,
];

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true)
            .with_transparent(false)
            .with_resizable(true)
            .with_maximized(false)
            .with_drag_and_drop(true)
            .with_inner_size([DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_SIZE, MIN_WINDOW_SIZE]),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(WaqlApp::new(cc)))),
    )
}

/// WAQL 工具应用程序主结构
struct WaqlApp {
    /// WAAPI 客户端实例
    client: WaapiClient,
    /// 用户输入的 WAQL 代码
    code: String,
    /// 查询执行结果或错误信息
    result: String,
    /// 当前选择的代码编辑器主题
    theme: ColorTheme,
    /// WAQL 语法定义
    syntax: Syntax,
    /// 代码自动补全器
    completer: Completer,
    /// 编辑器字体大小
    fontsize: f32,
}

impl Default for WaqlApp {
    fn default() -> Self {
        let syntax = waql_syntax();
        let mut completer = Completer::new_with_syntax(&syntax).with_user_words();
        for word in WAAPI_PROPERTIES.iter().chain(WAAPI_ACCESSORS.iter()) {
            completer.push_word(word);
        }

        Self {
            client: WaapiClient::default(),
            code: String::new(),
            result: String::new(),
            theme: ColorTheme::GRUVBOX,
            syntax: syntax.clone(),
            completer: completer,
            fontsize: DEFAULT_FONT_SIZE,
        }
    }
}

impl WaqlApp {
    /// 创建新的 WaqlApp 实例
    fn new(_cc: &CreationContext) -> Self {
        Self::default()
    }

    /// 执行 WAQL 查询并更新结果
    fn execute_query(&mut self) {
        let mut query = self.code.trim();
        let mut options: Option<Value> = None;
        if query.is_empty() {
            self.result = String::from("Please enter a WAQL statement.");
            return;
        }
        
        // 如果 | 包含在 WAQL 语句中，说明是包含 options 的复杂查询
        // 切分语句和选项部分
        if let Some((query_part, options_part)) = query.split_once('|') {
            query = query_part.trim();
            let options_str = options_part.trim();
            options = if options_str.is_empty() {
                None
            } else {
                Some(json!({
                    "return": options_str
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                }))
            };
        }

        match self.client.waql_query(query, options) {
            Ok(result) => {
                self.result = to_string_pretty(&result)
                    .unwrap_or_else(|_| "Failed to format result".to_string());
            }
            Err(e) => {
                self.result = format!("Error: {}", e);
            }
        }
    }

    /// 渲染主题选择面板
    fn render_theme_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("theme_picker").show(ctx, |ui| {
            ui.heading("Theme");
            egui::ScrollArea::both().show(ui, |ui| {
                for theme in THEMES.iter() {
                    if ui
                        .selectable_value(&mut self.theme, *theme, theme.name())
                        .clicked()
                    {
                        // 根据主题自动切换明暗模式
                        let visuals = if theme.is_dark() {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        };
                        ctx.set_visuals(visuals);
                    }
                }
            });
        });
    }

    /// 渲染代码输入编辑器
    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        // 提取需要的字段避免借用冲突
        let fontsize = self.fontsize;
        let syntax = &self.syntax;
        let theme = &self.theme;

        ui.horizontal(|h| {
            self.completer.show_on_text_widget(h, syntax, theme, |ui| {
                TextEdit::singleline(&mut self.code)
                    .hint_text(INPUT_HINT_TEXT)
                    .font(egui::FontId::monospace(fontsize))
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .layouter(&mut |ui: &egui::Ui, text: &dyn TextBuffer, _| {
                        let mut layout_job = egui::text::LayoutJob::default();
                        let font_id = egui::FontId::monospace(fontsize);

                        // 语法高亮
                        for token in Token::default().tokens(syntax, text.as_str()) {
                            let color = theme.type_color(token.ty());
                            let format = egui::text::TextFormat::simple(font_id.clone(), color);
                            layout_job.append(token.buffer(), 0.0, format);
                        }

                        ui.fonts_mut(|f| f.layout_job(layout_job))
                    })
                    .show(ui)
            });
        });
    }
}

impl eframe::App for WaqlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 左侧主题选择面板
        self.render_theme_panel(ctx);

        // 中央主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 代码输入编辑器
            self.render_code_editor(ui);

            // 检测回车键执行查询
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_query();
            }

            ui.separator();

            // 水平布局
            ui.horizontal(|ui| {
                // 运行按钮
                if ui.button("Run").clicked() {
                    self.execute_query();
                }

                // 清空按钮
                if ui.button("Clear").clicked() {
                    self.result.clear();
                }
            });

            ui.separator();

            // 结果显示区域
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.label(&self.result);
                });
        });
    }
}
