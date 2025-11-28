#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{self, CreationContext, egui};
use egui::{TextBuffer, TextEdit};
use egui_code_editor::{ColorTheme, Completer, Syntax, Token};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
const CONFIG_FILE_NAME: &str = "user_data.json";

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

/// 用户配置结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserConfig {
    /// 保存的 WAQL 语句列表
    saved_queries: Vec<String>,
    /// 选择的主题名称
    theme_name: String,
    /// 字体大小
    fontsize: f32,
    /// 自定义关键词列表
    custom_keywords: Vec<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            saved_queries: Vec::new(),
            theme_name: "GRUVBOX".to_string(),
            fontsize: DEFAULT_FONT_SIZE,
            custom_keywords: Vec::new(),
        }
    }
}

impl UserConfig {
    /// 从文件加载配置
    fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<UserConfig>(&content) {
                return config;
            }
        }
        Self::default()
    }

    /// 保存配置到文件
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, json)?;
        Ok(())
    }

    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        path.pop(); // 移除可执行文件名
        path.push(CONFIG_FILE_NAME);
        path
    }
}

/// WAQL 工具应用程序主结构
struct WaqlApp {
    /// WAAPI 客户端实例
    client: WaapiClient,
    /// 用户输入的 WAQL 代码
    code: String,
    /// 查询执行结果或错误信息
    result: String,
    /// 解析后的表格数据（列名和行数据）
    table_data: Option<(Vec<String>, Vec<HashMap<String, String>>)>,
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
            client: WaapiClient::default(),
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
    fn new(_cc: &CreationContext) -> Self {
        Self::default()
    }

    /// 导出结果到 CSV 文件
    fn export_to_csv(&self) {
        if let Some((columns, rows)) = &self.table_data {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("waql_results.csv")
                .add_filter("CSV Files", &["csv"])
                .save_file()
            {
                if let Err(e) = self.write_csv(&path, columns, rows) {
                    eprintln!("Failed to export CSV: {}", e);
                }
            }
        }
    }

    /// 将数据写入 CSV 文件
    fn write_csv(
        &self,
        path: &std::path::Path,
        columns: &[String],
        rows: &[HashMap<String, String>],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_path(path)?;

        // 写入表头
        writer.write_record(columns)?;

        // 写入数据行
        for row in rows {
            let record: Vec<&str> = columns
                .iter()
                .map(|col| row.get(col).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// 执行 WAQL 查询并更新结果
    fn execute_query(&mut self) {
        let mut query = self.code.trim();
        let mut options: Option<Value> = None;
        if query.is_empty() {
            self.result = String::from("Please enter a WAQL statement.");
            self.has_error = true;
            self.table_data = None;
            self.status_message = "No query entered".to_string();
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
                self.has_error = false;
                self.result = to_string_pretty(&result)
                    .unwrap_or_else(|_| "Failed to format result".to_string());

                // 解析返回数据为表格格式
                if let Some(return_array) = result.get("return").and_then(|v| v.as_array()) {
                    let count = return_array.len();
                    if !return_array.is_empty() {
                        // 提取所有可能的列名（从所有对象的键中收集）
                        let mut columns = Vec::new();
                        let mut columns_set = std::collections::HashSet::new();

                        for item in return_array {
                            if let Some(obj) = item.as_object() {
                                for key in obj.keys() {
                                    if columns_set.insert(key.clone()) {
                                        columns.push(key.clone());
                                    }
                                }
                            }
                        }

                        // 转换数据行
                        let mut rows = Vec::new();
                        for item in return_array {
                            if let Some(obj) = item.as_object() {
                                let mut row = HashMap::new();
                                for col in &columns {
                                    let value = obj
                                        .get(col)
                                        .map(|v| match v {
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            Value::Bool(b) => b.to_string(),
                                            Value::Null => "null".to_string(),
                                            _ => serde_json::to_string(v).unwrap_or_default(),
                                        })
                                        .unwrap_or_default();
                                    row.insert(col.clone(), value);
                                }
                                rows.push(row);
                            }
                        }

                        self.table_data = Some((columns, rows));
                        self.status_message = format!("Query successful - {} result(s)", count);
                    } else {
                        self.table_data = None;
                        self.status_message.clear();
                    }
                } else {
                    self.table_data = None;
                    self.status_message.clear();
                }
            }
            Err(e) => {
                self.result = format!("Error: {}", e);
                self.has_error = true;
                self.table_data = None;
                self.status_message = "Query failed".to_string();
            }
        }
    }

    /// 渲染代码输入编辑器
    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        // 提取需要的字段避免借用冲突
        let fontsize = self.config.fontsize;
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
        // 底部配置面板
        if self.show_config_panel {
            egui::TopBottomPanel::bottom("config_panel")
                .resizable(true)
                .default_height(300.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // 主题选择区域
                        ui.group(|ui| {
                            ui.heading("Theme");
                            ui.separator();
                            ui.horizontal_wrapped(|ui| {
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

                                        // 保存主题到配置
                                        self.config.theme_name = theme.name().to_string();
                                        let _ = self.config.save();
                                    }
                                }
                            });
                        });

                        ui.separator();

                        // WAQL 语句列表区域
                        ui.group(|ui| {
                            ui.heading("Saved Queries");
                            ui.separator();

                            let mut query_to_load: Option<String> = None;
                            let mut query_to_delete: Option<usize> = None;

                            for (index, query) in self.config.saved_queries.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    if ui.button("Load").clicked() {
                                        query_to_load = Some(query.clone());
                                    }
                                    ui.label(query);
                                    if ui.button("❌").clicked() {
                                        query_to_delete = Some(index);
                                    }
                                });
                            }

                            if let Some(query) = query_to_load {
                                self.code = query;
                            }

                            if let Some(index) = query_to_delete {
                                self.config.saved_queries.remove(index);
                                let _ = self.config.save();
                            }
                        });

                        ui.separator();

                        // 自定义关键词区域
                        ui.group(|ui| {
                            ui.heading("Custom Keywords");
                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Add:");
                                ui.text_edit_singleline(&mut self.custom_keyword);
                                if ui.button("Add").clicked() {
                                    let keyword = self.custom_keyword.trim().to_string();
                                    if !keyword.is_empty()
                                        && !self.config.custom_keywords.contains(&keyword)
                                    {
                                        self.config.custom_keywords.push(keyword.clone());
                                        self.completer.push_word(&keyword);
                                        self.custom_keyword.clear();
                                        let _ = self.config.save();
                                    }
                                }
                            });

                            ui.separator();

                            let mut keyword_to_delete: Option<usize> = None;
                            for (index, keyword) in self.config.custom_keywords.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(keyword);
                                    if ui.button("❌").clicked() {
                                        keyword_to_delete = Some(index);
                                    }
                                });
                            }

                            if let Some(index) = keyword_to_delete {
                                self.config.custom_keywords.remove(index);
                                let _ = self.config.save();
                            }
                        });
                    });
                });
        }

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
                // 运行按钮 - 仅在输入框有内容时启用
                let has_code = !self.code.trim().is_empty();
                if ui.add_enabled(has_code, egui::Button::new("Run WAQL")).clicked() {
                    self.execute_query();
                }

                // 保存按钮 - 仅在输入框有内容时启用
                if ui.add_enabled(has_code, egui::Button::new("Save WAQL")).clicked() {
                    let query = self.code.trim().to_string();
                    if !query.is_empty() && !self.config.saved_queries.contains(&query) {
                        self.config.saved_queries.push(query);
                        // 保存配置到文件
                        if let Err(e) = self.config.save() {
                            self.result = format!("Failed to save config: {}", e);
                        }
                    }
                }

                // 导出 CSV 按钮 - 仅在有表格数据时启用
                if ui.add_enabled(self.table_data.is_some(), egui::Button::new("Export CSV")).clicked() {
                    self.export_to_csv();
                }

                // 清空按钮 - 仅在有结果时启用
                let has_results = !self.result.is_empty() || self.table_data.is_some();
                if ui.add_enabled(has_results, egui::Button::new("Clear Results")).clicked() {
                    self.result.clear();
                    self.table_data = None;
                    self.has_error = false;
                    self.status_message.clear();
                }

                ui.separator();

                // 显示/隐藏配置按钮
                let config_button_text = if self.show_config_panel {
                    "Hide Config"
                } else {
                    "Show Config"
                };
                if ui.button(config_button_text).clicked() {
                    self.show_config_panel = !self.show_config_panel;
                }

                // 状态消息显示
                if !self.status_message.is_empty() {
                    ui.separator();
                    let color = if self.has_error {
                        egui::Color32::RED
                    } else {
                        egui::Color32::GREEN
                    };
                    ui.colored_label(color, &self.status_message);
                }
            });

            ui.separator();

            // 结果显示区域
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if self.has_error {
                        // 显示错误信息
                        ui.colored_label(egui::Color32::RED, &self.result);
                    } else if let Some((columns, rows)) = &self.table_data {
                        // 显示表格
                        use egui_extras::{Column, TableBuilder};

                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::auto()) // 序号列
                            .min_scrolled_height(0.0);

                        let table = columns.iter().fold(table, |t, _| t.column(Column::auto()));

                        table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("#");
                                });
                                for col in columns {
                                    header.col(|ui| {
                                        ui.strong(col);
                                    });
                                }
                            })
                            .body(|mut body| {
                                for (index, row) in rows.iter().enumerate() {
                                    body.row(18.0, |mut row_ui| {
                                        row_ui.col(|ui| {
                                            ui.label((index + 1).to_string());
                                        });
                                        for col in columns {
                                            row_ui.col(|ui| {
                                                ui.label(
                                                    row.get(col).map(|s| s.as_str()).unwrap_or(""),
                                                );
                                            });
                                        }
                                    });
                                }
                            });
                    } else {
                        // 显示原始 JSON（当没有 return 数组或为空时）
                        ui.label(&self.result);
                    }
                });
        });
    }
}
