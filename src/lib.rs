//! WAQL Tool 库
//! 
//! 提供 WAQL 语法高亮、代码补全和查询执行功能

pub mod config;
pub mod query_executor;
mod waql;

pub use waql::waql_syntax;
pub use waql::WAAPI_ACCESSORS;
pub use waql::WAAPI_PROPERTIES;