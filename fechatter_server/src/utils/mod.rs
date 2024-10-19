pub(crate) mod jwt;
pub(crate) mod test_utils;

#[macro_export]
macro_rules! auth_user {
  ($user:expr) => {
    $crate::models::AuthUser {
      id: $user.id,
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
