//! WAQL Tool - Wwise WAQL 查询工具
//! 
//! 这是一个图形化的 WAQL (Wwise Authoring Query Language) 查询工具，
//! 提供语法高亮、代码补全和结果可视化功能。
//! 
//! # 主要功能
//! 
//! - WAQL 语法高亮和代码补全
//! - 查询结果表格展示
//! - 导出结果为 CSV 文件
//! - 保存常用查询语句
//! - 自定义关键词
//! - 多主题支持

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod query_executor;
mod ui;

use config::UserConfig;
use eframe::{self, CreationContext, egui};
use egui_code_editor::{ColorTheme, Completer, Syntax};
use query_executor::{QueryExecutor, TableData};
use ui::{
    render_code_editor, render_config_panel, render_control_buttons, render_results, THEMES,
};
use waql_tool::{waql_syntax, WAAPI_ACCESSORS, WAAPI_PROPERTIES};

// UI 常量
const APP_TITLE: &str = "Waql Tool";
const DEFAULT_WINDOW_WIDTH: f32 = 900.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;
const MIN_WINDOW_SIZE: f32 = 280.0;

/// 设置自定义字体
fn setup_custom_fonts(ctx: &egui::Context, fontsize: f32) {
    // 从默认字体开始
    let mut fonts = egui::FontDefinitions::default();

    // 加载 SIMKAI.TTF 字体文件
    fonts.font_data.insert(
        "simkai".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "fonts/SIMKAI.TTF"
        ))),
    );

    // 将 simkai 字体设置为 Proportional 字体族的第一优先级（用于界面文本）
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "simkai".to_owned());

    // Monospace 保持使用默认的等宽字体，不使用楷体
    // 这样代码编辑器会使用系统默认的等宽字体，显示效果更好

    // 应用字体配置
    ctx.set_fonts(fonts);
    
    // 设置字体大小
    update_font_size(ctx, fontsize);
}

/// 更新字体大小
fn update_font_size(ctx: &egui::Context, fontsize: f32) {
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(fontsize * 0.9),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(fontsize * 0.9),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(fontsize),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::monospace(fontsize),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(fontsize * 0.8),
    );
    ctx.set_style(style);
}

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
    /// 查询执行器
    executor: QueryExecutor,
    /// 用户输入的 WAQL 代码
    code: String,
    /// 查询执行结果或错误信息
    result: String,
    /// 解析后的表格数据
    table_data: Option<TableData>,
    /// 是否有错误
    has_error: bool,
    /// 当前选择的代码编辑器主题
    theme: ColorTheme,
    /// WAQL 语法定义
    syntax: Syntax,
    /// 代码自动补全器
    completer: Completer,
    /// 用户配置
    config: UserConfig,
    /// 自定义关键词输入框
    custom_keyword: String,
    /// 是否显示配置面板
    show_config_panel: bool,
    /// 状态消息
    status_message: String,
}

impl Default for WaqlApp {
    fn default() -> Self {
        let syntax = waql_syntax();
        let mut completer = Completer::new_with_syntax(&syntax).with_user_words();
        for word in WAAPI_PROPERTIES.iter().chain(WAAPI_ACCESSORS.iter()) {
            completer.push_word(word);
        }

        // 加载用户配置
        let config = UserConfig::load();

        // 加载自定义关键词到补全器
        for keyword in &config.custom_keywords {
            completer.push_word(keyword);
        }

        // 根据配置中的主题名称选择主题
        let theme = THEMES
            .iter()
            .find(|t| t.name() == config.theme_name)
            .copied()
            .unwrap_or(ColorTheme::GRUVBOX);

        Self {
            executor: QueryExecutor::new(),
            code: String::new(),
            result: String::new(),
            table_data: None,
            has_error: false,
            theme,
            syntax: syntax.clone(),
            completer,
            config,
            custom_keyword: String::new(),
            show_config_panel: false,
            status_message: String::new(),
        }
    }
}

impl WaqlApp {
    /// 创建新的 WaqlApp 实例
    fn new(cc: &CreationContext) -> Self {
        // 先加载配置以获取字体大小
        let config = UserConfig::load();
        // 设置自定义字体和大小
        setup_custom_fonts(&cc.egui_ctx, config.fontsize);
        Self::default()
    }

    /// 导出结果到 CSV 文件
    fn export_to_csv(&self) {
        if let Some(table_data) = &self.table_data {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("waql_results.csv")
                .add_filter("CSV Files", &["csv"])
                .save_file()
            {
                if let Err(e) = table_data.export_to_csv(&path) {
                    eprintln!("Failed to export CSV: {}", e);
                }
            }
        }
    }

    /// 执行 WAQL 查询并更新结果
    fn execute_query(&mut self) {
        match self.executor.execute(&self.code) {
            Ok(result) => {
                self.has_error = false;
                self.result = result.raw_json;
                self.table_data = result.table_data;
                self.status_message = if result.count > 0 {
                    format!("查询成功 - {} 条结果", result.count)
                } else {
                    String::new()
                };
            }
            Err(e) => {
                self.result = e;
                self.has_error = true;
                self.table_data = None;
                self.status_message = "查询失败".to_string();
            }
        }
    }
}

impl eframe::App for WaqlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 底部配置面板
        if self.show_config_panel {
            egui::TopBottomPanel::bottom("config_panel")
                .resizable(true)
                .default_height(300.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let actions = render_config_panel(
                            ui,
                            &mut self.config,
                            &mut self.theme,
                            &mut self.custom_keyword,
                            &mut self.completer,
                            &mut self.code,
                            ctx,
                        );

                        // 处理配置面板操作
                        if actions.fontsize_changed {
                            // 字体大小改变时，重新应用字体设置
                            update_font_size(ctx, self.config.fontsize);
                        }

                        if actions.save_config {
                            let _ = self.config.save();
                        }

                        if let Some(index) = actions.remove_query_index {
                            self.config.remove_saved_query(index);
                            let _ = self.config.save();
                        }

                        if let Some(index) = actions.remove_keyword_index {
                            self.config.remove_custom_keyword(index);
                            let _ = self.config.save();
                        }
                    });
                });
        }

        // 中央主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 代码输入编辑器
            render_code_editor(
                ui,
                &mut self.code,
                &mut self.completer,
                &self.syntax,
                &self.theme,
                self.config.fontsize,
            );

            // 检测回车键执行查询
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_query();
            }

            ui.separator();

            // 控制按钮栏
            let has_code = !self.code.trim().is_empty();
            let has_results = !self.result.is_empty() || self.table_data.is_some();
            let actions = render_control_buttons(
                ui,
                has_code,
                has_results,
                self.table_data.is_some(),
                &mut self.show_config_panel,
                &self.status_message,
                self.has_error,
            );

            // 处理控制按钮操作
            if actions.run_query {
                self.execute_query();
            }

            if actions.save_query {
                let query = self.code.trim().to_string();
                if self.config.add_saved_query(query) {
                    if let Err(e) = self.config.save() {
                        self.result = format!("保存配置失败: {}", e);
                    }
                }
            }

            if actions.export_csv {
                self.export_to_csv();
            }

            if actions.clear_results {
                self.result.clear();
                self.table_data = None;
                self.has_error = false;
                self.status_message.clear();
            }

            ui.separator();

            // 结果显示区域
            render_results(ui, &self.result, &self.table_data, self.has_error);
        });
    }
}
