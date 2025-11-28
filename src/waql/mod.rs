//! WAQL 语法和属性定义模块
//! 
//! 包含 WAQL 语法定义、WAAPI 属性和访问器列表

mod properties;
mod syntax;

pub use properties::WAAPI_ACCESSORS;
pub use properties::WAAPI_PROPERTIES;
pub use syntax::waql_syntax;