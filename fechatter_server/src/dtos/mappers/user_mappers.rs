use fechatter_core::models::User;

/// 用户映射器 - 已被新的DTOs转换框架取代
///
/// 注意：这个映射器已经过时，新的架构使用 core::Converter trait 进行类型安全的转换
/// 请使用 fechatter_server::dtos::core 模块中的转换框架
pub struct UserMapper;

impl UserMapper {
  /// 已废弃 - 请使用新的转换框架
  #[deprecated(note = "使用 RequestDto::to_domain() 方法代替")]
  pub fn request_to_domain<T>(_request: &T) -> Result<fechatter_core::models::CreateUser, String> {
    Err("此映射器已废弃，请使用新的DTOs转换框架".to_string())
  }

  /// 已废弃 - 请使用新的转换框架
  #[deprecated(note = "使用 ResponseDto::from_domain() 方法代替")]
  pub fn domain_to_response(_user: &User) -> Result<(), String> {
    Err("此映射器已废弃，请使用新的DTOs转换框架".to_string())
  }
}
