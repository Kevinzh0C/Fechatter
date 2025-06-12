// 契约模块，定义所有的服务接口
pub mod repositories;
pub mod services;

// 基础设施服务接口契约
pub mod infrastructure;

// 事件接口契约
pub mod events;

// 重新导出接口契约
pub use events::*;
pub use infrastructure::*;
pub use repositories::*;
pub use services::*;

// 明确导出AuthContext以解决可见性问题
pub use services::AuthContext;
