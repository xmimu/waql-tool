//! WAQL 查询执行器模块
//! 
//! 负责执行 WAQL 查询并处理结果

use serde_json::{json, to_string_pretty, Value};
use std::collections::HashMap;
use waapi_rs::WaapiClient;

/// WAQL 查询执行结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 原始 JSON 结果
    pub raw_json: String,
    /// 解析后的表格数据（列名和行数据）
    pub table_data: Option<TableData>,
    /// 结果数量
    pub count: usize,
}

/// 表格数据结构
#[derive(Debug, Clone)]
pub struct TableData {
    /// 列名列表
    pub columns: Vec<String>,
    /// 行数据列表
    pub rows: Vec<HashMap<String, String>>,
}

impl TableData {
    /// 导出为 CSV 格式
    /// 
    /// # Errors
    /// 
    /// 如果写入 CSV 失败，返回错误
    pub fn export_to_csv(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_path(path)?;

        // 写入表头
        writer.write_record(&self.columns)?;

        // 写入数据行
        for row in &self.rows {
            let record: Vec<&str> = self
                .columns
                .iter()
                .map(|col| row.get(col).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }
}

/// WAQL 查询执行器
pub struct QueryExecutor {
    client: WaapiClient,
}

impl Default for QueryExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryExecutor {
    /// 创建新的查询执行器
    pub fn new() -> Self {
        Self {
            client: WaapiClient::default(),
        }
    }

    /// 执行 WAQL 查询
    /// 
    /// # Arguments
    /// 
    /// * `code` - WAQL 查询语句，可以包含 options（用 | 分隔）
    /// 
    /// # Returns
    /// 
    /// 返回查询结果或错误信息
    pub fn execute(&mut self, code: &str) -> Result<QueryResult, String> {
        let code = code.trim();
        
        if code.is_empty() {
            return Err("请输入 WAQL 查询语句".to_string());
        }

        let (query, options) = self.parse_query(code);

        match self.client.waql_query(query, options) {
            Ok(result) => {
                // 将 Map 转换为 Value
                let result_value = Value::Object(result);
                
                let raw_json = to_string_pretty(&result_value)
                    .unwrap_or_else(|_| "格式化结果失败".to_string());

                let table_data = Self::parse_table_data(&result_value);
                let count = table_data.as_ref().map(|t| t.rows.len()).unwrap_or(0);

                Ok(QueryResult {
                    raw_json,
                    table_data,
                    count,
                })
            }
            Err(e) => Err(format!("查询失败: {}", e)),
        }
    }

    /// 解析 WAQL 查询语句和选项
    /// 
    /// 如果查询语句包含 `|`，则分割为查询部分和选项部分
    fn parse_query<'a>(&self, code: &'a str) -> (&'a str, Option<Value>) {
        if let Some((query_part, options_part)) = code.split_once('|') {
            let query = query_part.trim();
            let options_str = options_part.trim();
            
            let options = if options_str.is_empty() {
                None
            } else {
                Some(json!({
                    "return": options_str
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                }))
            };
            
            (query, options)
        } else {
            (code, None)
        }
    }

    /// 从 JSON 结果中解析表格数据
    fn parse_table_data(result: &Value) -> Option<TableData> {
        let return_array = result.get("return")?.as_array()?;

        if return_array.is_empty() {
            return None;
        }

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
                        .map(|v| Self::value_to_string(v))
                        .unwrap_or_default();
                    row.insert(col.clone(), value);
                }
                rows.push(row);
            }
        }

        Some(TableData { columns, rows })
    }

    /// 将 JSON Value 转换为字符串
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_without_options() {
        let executor = QueryExecutor::new();
        let (query, options) = executor.parse_query("$ from type Sound");
        assert_eq!(query, "$ from type Sound");
        assert!(options.is_none());
    }

    #[test]
    fn test_parse_query_with_options() {
        let executor = QueryExecutor::new();
        let (query, options) = executor.parse_query("$ from type Sound | name id");
        assert_eq!(query, "$ from type Sound");
        assert!(options.is_some());
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(QueryExecutor::value_to_string(&json!("test")), "test");
        assert_eq!(QueryExecutor::value_to_string(&json!(42)), "42");
        assert_eq!(QueryExecutor::value_to_string(&json!(true)), "true");
        assert_eq!(QueryExecutor::value_to_string(&json!(null)), "null");
    }
}
