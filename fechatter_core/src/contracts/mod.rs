// 仓储接口契约
pub mod repositories;

// 服务接口契约
pub mod services;

// 基础设施服务接口契约
pub mod infrastructure;

// 重新导出接口契约
pub use infrastructure::*;
pub use repositories::*;
pub use services::*;

// 明确导出AuthContext以解决可见性问题
pub use services::AuthContext;
