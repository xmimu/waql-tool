//! UI 渲染模块
//! 
//! 包含各种 UI 组件的渲染逻辑

use crate::config::UserConfig;
use crate::query_executor::TableData;
use egui::{TextBuffer, TextEdit};
use egui_code_editor::{ColorTheme, Completer, Syntax, Token};

/// 输入提示文本
const INPUT_HINT_TEXT: &str = "Enter the WAQL statement here.";

/// 可用的代码编辑器主题列表
pub const THEMES: [ColorTheme; 8] = [
    ColorTheme::AYU,
    ColorTheme::AYU_MIRAGE,
    ColorTheme::AYU_DARK,
    ColorTheme::GITHUB_DARK,
    ColorTheme::GITHUB_LIGHT,
    ColorTheme::GRUVBOX,
    ColorTheme::GRUVBOX_LIGHT,
    ColorTheme::SONOKAI,
];

/// 渲染代码输入编辑器
pub fn render_code_editor(
    ui: &mut egui::Ui,
    code: &mut String,
    completer: &mut Completer,
    syntax: &Syntax,
    theme: &ColorTheme,
    fontsize: f32,
) {
    ui.horizontal(|h| {
        completer.show_on_text_widget(h, syntax, theme, |ui| {
            TextEdit::singleline(code)
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

/// 渲染配置面板
pub fn render_config_panel(
    ui: &mut egui::Ui,
    config: &mut UserConfig,
    theme: &mut ColorTheme,
    custom_keyword: &mut String,
    completer: &mut Completer,
    code: &mut String,
    ctx: &egui::Context,
) -> ConfigPanelActions {
    let mut actions = ConfigPanelActions::default();

    // 主题选择区域
    ui.group(|ui| {
        ui.heading("Theme");
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for available_theme in THEMES.iter() {
                if ui
                    .selectable_value(theme, *available_theme, available_theme.name())
                    .clicked()
                {
                    // 根据主题自动切换明暗模式
                    let visuals = if available_theme.is_dark() {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ctx.set_visuals(visuals);

                    // 保存主题到配置
                    config.theme_name = available_theme.name().to_string();
                    actions.save_config = true;
                }
            }
        });
    });

    ui.separator();

    // 字体大小调节区域
    ui.group(|ui| {
        ui.heading("Font Size");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Size:");
            if ui.add(egui::Slider::new(&mut config.fontsize, 10.0..=32.0).text("pt")).changed() {
                actions.fontsize_changed = true;
                actions.save_config = true;
            }
            if ui.button("Reset").clicked() {
                config.fontsize = 18.0;
                actions.fontsize_changed = true;
                actions.save_config = true;
            }
        });
    });

    ui.separator();

    // WAQL 语句列表区域
    ui.group(|ui| {
        ui.heading("Saved Queries");
        ui.separator();

        for (index, query) in config.saved_queries.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.button("Load").clicked() {
                    *code = query.clone();
                }
                ui.label(query);
                if ui.button("❌").clicked() {
                    actions.remove_query_index = Some(index);
                }
            });
        }
    });

    ui.separator();

    // 自定义关键词区域
    ui.group(|ui| {
        ui.heading("Custom Keywords");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Add:");
            ui.text_edit_singleline(custom_keyword);
            if ui.button("Add").clicked() {
                let keyword = custom_keyword.trim().to_string();
                if config.add_custom_keyword(keyword.clone()) {
                    completer.push_word(&keyword);
                    custom_keyword.clear();
                    actions.save_config = true;
                }
            }
        });

        ui.separator();

        for (index, keyword) in config.custom_keywords.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(keyword);
                if ui.button("❌").clicked() {
                    actions.remove_keyword_index = Some(index);
                }
            });
        }
    });

    actions
}

/// 配置面板操作结果
#[derive(Default)]
pub struct ConfigPanelActions {
    /// 是否需要保存配置
    pub save_config: bool,
    /// 需要删除的查询索引
    pub remove_query_index: Option<usize>,
    /// 需要删除的关键词索引
    pub remove_keyword_index: Option<usize>,
    /// 字体大小是否改变
    pub fontsize_changed: bool,
}

/// 渲染控制按钮栏
pub fn render_control_buttons(
    ui: &mut egui::Ui,
    has_code: bool,
    has_results: bool,
    has_table_data: bool,
    show_config_panel: &mut bool,
    status_message: &str,
    has_error: bool,
) -> ControlButtonActions {
    let mut actions = ControlButtonActions::default();

    ui.horizontal(|ui| {
        // 运行按钮
        if ui.add_enabled(has_code, egui::Button::new("Run WAQL")).clicked() {
            actions.run_query = true;
        }

        // 保存按钮
        if ui.add_enabled(has_code, egui::Button::new("Save WAQL")).clicked() {
            actions.save_query = true;
        }

        // 导出 CSV 按钮
        if ui.add_enabled(has_table_data, egui::Button::new("Export CSV")).clicked() {
            actions.export_csv = true;
        }

        // 清空按钮
        if ui.add_enabled(has_results, egui::Button::new("Clear Results")).clicked() {
            actions.clear_results = true;
        }

        ui.separator();

        // 显示/隐藏配置按钮
        let config_button_text = if *show_config_panel {
            "Hide Config"
        } else {
            "Show Config"
        };
        if ui.button(config_button_text).clicked() {
            *show_config_panel = !*show_config_panel;
        }

        // 状态消息显示
        if !status_message.is_empty() {
            ui.separator();
            let color = if has_error {
                egui::Color32::RED
            } else {
                egui::Color32::GREEN
            };
            ui.colored_label(color, status_message);
        }
    });

    actions
}

/// 控制按钮操作结果
#[derive(Default)]
pub struct ControlButtonActions {
    /// 是否运行查询
    pub run_query: bool,
    /// 是否保存查询
    pub save_query: bool,
    /// 是否导出 CSV
    pub export_csv: bool,
    /// 是否清空结果
    pub clear_results: bool,
}

/// 渲染结果显示区域
pub fn render_results(
    ui: &mut egui::Ui,
    result: &str,
    table_data: &Option<TableData>,
    has_error: bool,
) {
    egui::ScrollArea::both()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if has_error {
                // 显示错误信息
                ui.colored_label(egui::Color32::RED, result);
            } else if let Some(data) = table_data {
                // 显示表格
                render_table(ui, data);
            } else {
                // 显示原始 JSON
                ui.label(result);
            }
        });
}

/// 渲染数据表格
fn render_table(ui: &mut egui::Ui, data: &TableData) {
    use egui_extras::{Column, TableBuilder};

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto()) // 序号列
        .min_scrolled_height(0.0);

    let table = data.columns.iter().fold(table, |t, _| t.column(Column::auto()));

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("#");
            });
            for col in &data.columns {
                header.col(|ui| {
                    ui.strong(col);
                });
            }
        })
        .body(|mut body| {
            for (index, row) in data.rows.iter().enumerate() {
                body.row(18.0, |mut row_ui| {
                    row_ui.col(|ui| {
                        ui.label((index + 1).to_string());
                    });
                    for col in &data.columns {
                        row_ui.col(|ui| {
                            ui.label(row.get(col).map(|s| s.as_str()).unwrap_or(""));
                        });
                    }
                });
            }
        });
}
