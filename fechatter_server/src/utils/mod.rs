pub mod jwt;
pub mod test_utils;
pub mod token;

#[macro_export]
macro_rules! auth_user {
  ($user:expr) => {
    $crate::models::AuthUser {
      id: $user.id,
      workspace_id: $user.workspace_id,
      fullname: $user.fullname.clone(),
      email: $user.email.clone(),
      status: $user.status.clone(),
      created_at: $user.created_at,
    }
  };
}

#[macro_export]
macro_rules! create_user {
  ($fullname:expr, $email:expr, $password:expr) => {
    $crate::models::CreateUser {
      fullname: $fullname.to_string(),
      email: $email.to_string(),
      password: $password.to_string(),
    }
  };
}
