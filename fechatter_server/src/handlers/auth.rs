//! # Authentication Handlers - Simplified Implementation
//!
//! **Responsibility**: HTTP authentication handling, delegating to application services
//! **Principles**: Simple design, single responsibility

use crate::dtos::core::ApiResponse;
use crate::dtos::models::requests::auth::{LoginRequest, RegisterRequest};
use crate::dtos::models::responses::auth::{
    LoginResponse, LogoutResponse, RefreshTokenResponse, RegisterResponse,
};
use crate::{error::AppError, AppState};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Utc};
use fechatter_core::models::{AuthUser, CreateUser};
use fechatter_core::{
    contracts::AuthContext,
    models::jwt::ACCESS_TOKEN_EXPIRATION,
    models::jwt::{LogoutService, RefreshTokenService, SigninService, SignupService},
    SigninUser,
};
use std::time::Instant;
use tracing::instrument;
use validator::Validate;

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Extract request ID from headers
fn extract_request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string())
}

/// Set refresh token cookie
fn set_refresh_token_cookie(
    headers: &mut HeaderMap,
    token_str: &str,
    expires_at: &DateTime<Utc>,
) -> Result<(), AppError> {
    let mut cookie = Cookie::new("refresh_token", token_str.to_string());
    cookie.set_http_only(true);

    // Fix for development environment - use Lax instead of None to avoid browser security warnings
    // In development (HTTP), SameSite=None requires Secure=true which we can't set
    cookie.set_secure(false);
    cookie.set_same_site(Some(SameSite::Lax)); // Changed from None to Lax for HTTP environments
    cookie.set_path("/");

    let now = Utc::now();
    let duration_seconds = expires_at.signed_duration_since(now).num_seconds();
    let time_str = expires_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    let cookie_str = if duration_seconds <= 0 {
        format!("{}; Max-Age=0; Expires={}", cookie.to_string(), time_str)
    } else {
        format!(
            "{}; Max-Age={}; Expires={}",
            cookie.to_string(),
            duration_seconds,
            time_str
        )
    };

    match HeaderValue::from_str(&cookie_str) {
        Ok(hv) => {
            headers.insert(header::SET_COOKIE, hv);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create header value from cookie: {}", e);
            Err(AppError::Internal(
                "Failed to set refresh token cookie".to_string(),
            ))
        }
    }
}

/// Clear refresh token cookie
fn clear_refresh_token_cookie(headers: &mut HeaderMap) -> Result<(), AppError> {
    let mut cookie = Cookie::new("refresh_token", "");
    cookie.set_http_only(true);

    // Fix for development environment - use Lax instead of None to avoid browser security warnings
    cookie.set_secure(false);
    cookie.set_same_site(Some(SameSite::Lax)); // Changed from None to Lax for HTTP environments
    cookie.set_path("/");

    let cookie_str = format!(
        "{}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
        cookie.to_string()
    );

    match HeaderValue::from_str(&cookie_str) {
        Ok(hv) => {
            headers.insert(header::SET_COOKIE, hv);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create header value from cookie: {}", e);
            Err(AppError::Internal(
                "Failed to clear refresh token cookie".to_string(),
            ))
        }
    }
}

/// Extract AuthContext from request headers
fn extract_auth_context(headers: &HeaderMap) -> Option<AuthContext> {
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    Some(AuthContext {
        user_agent,
        ip_address,
    })
}

// =============================================================================
// HANDLERS
// =============================================================================

/// User Signup Handler
#[instrument(skip_all, fields(email = %request.email))]
pub async fn signup_handler(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let start_time = Instant::now();
    let request_id = extract_request_id(&headers);
    let auth_context = extract_auth_context(&headers);

    if let Err(e) = request.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                crate::dtos::core::ApiError {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Invalid input".to_string(),
                    details: Some(e.to_string()),
                    field: None,
                    stack: vec![],
                    suggestion: Some("Please check your input and try again".to_string()),
                    help_url: Some("/docs/auth/signup".to_string()),
                },
                request_id,
            )),
        )
            .into_response());
    }

    let create_user = CreateUser {
        fullname: request.fullname,
        email: request.email,
        password: request.password,
        workspace: request
            .workspace_name
            .unwrap_or_else(|| "Default".to_string()),
    };

    let auth_service =
        crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
    match auth_service.signup(&create_user, auth_context).await {
        Ok(tokens) => {
            let mut response_headers = HeaderMap::new();
            set_refresh_token_cookie(
                &mut response_headers,
                &tokens.refresh_token.token,
                &tokens.refresh_token.expires_at,
            )?;

            let user = if let Ok(claims) = state
                .token_manager()
                .internal_verify_token(&tokens.access_token)
            {
                fechatter_core::User {
                    id: fechatter_core::UserId(claims.id.into()),
                    fullname: create_user.fullname.clone(),
                    email: create_user.email.clone(),
                    password_hash: Some(String::new()),
                    status: fechatter_core::models::UserStatus::Active,
                    created_at: chrono::Utc::now(),
                    workspace_id: fechatter_core::WorkspaceId(claims.workspace_id.into()),
                    // Profile fields - set defaults for new users
                    phone: None,
                    title: None,
                    department: None,
                    avatar_url: None,
                    bio: None,
                    timezone: None,
                    language: None,
                    last_active_at: Some(chrono::Utc::now()),
                }
            } else {
                return Err(AppError::Internal(
                    "Failed to decode token claims".to_string(),
                ));
            };

            let workspace = Some(fechatter_core::Workspace {
                id: user.workspace_id.into(),
                name: create_user.workspace,
                owner_id: user.id.into(),
                created_at: chrono::Utc::now(),
            });

            let response = RegisterResponse {
                user,
                workspace,
                message: "User registered successfully".to_string(),
                email_verification_required: false,
                created_at: chrono::Utc::now(),
            };

            let api_response = ApiResponse::success(response, request_id)
                .with_duration(start_time.elapsed().as_millis() as u64);

            Ok((StatusCode::CREATED, response_headers, Json(api_response)).into_response())
        }
        Err(fechatter_core::error::CoreError::UserAlreadyExists(msg)) => Ok((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error(
                crate::dtos::core::ApiError {
                    code: "USER_EXISTS".to_string(),
                    message: msg,
                    details: None,
                    field: Some("email".to_string()),
                    stack: vec![],
                    suggestion: Some("Please use a different email or login instead".to_string()),
                    help_url: Some("/docs/auth/signup".to_string()),
                },
                request_id,
            )),
        )
            .into_response()),
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                crate::dtos::core::ApiError::from(e),
                request_id,
            )),
        )
            .into_response()),
    }
}

/// User Signin Handler
pub async fn signin_handler(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let start_time = Instant::now();
    let request_id = extract_request_id(&headers);
    let auth_context = extract_auth_context(&headers);

    if let Err(e) = request.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                crate::dtos::core::ApiError {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Invalid input".to_string(),
                    details: Some(e.to_string()),
                    field: None,
                    stack: vec![],
                    suggestion: Some("Please check your email and password".to_string()),
                    help_url: Some("/docs/auth/login".to_string()),
                },
                request_id,
            )),
        )
            .into_response());
    }

    let signin_user = SigninUser::new(&request.email, &request.password);

    let auth_service =
        crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
    match auth_service.signin(&signin_user, auth_context).await? {
        Some(tokens) => {
            let mut response_headers = HeaderMap::new();
            set_refresh_token_cookie(
                &mut response_headers,
                &tokens.refresh_token.token,
                &tokens.refresh_token.expires_at,
            )?;

            let (user, workspace) = if let Ok(claims) = state
                .token_manager()
                .internal_verify_token(&tokens.access_token)
            {
                let user = fechatter_core::User {
                    id: fechatter_core::UserId(claims.id.into()),
                    fullname: claims.fullname.clone(),
                    email: claims.email.clone(),
                    password_hash: Some(String::new()),
                    status: fechatter_core::models::UserStatus::Active,
                    created_at: chrono::Utc::now(),
                    workspace_id: fechatter_core::WorkspaceId(claims.workspace_id.into()),
                    // Profile fields - set defaults for new users
                    phone: None,
                    title: None,
                    department: None,
                    avatar_url: None,
                    bio: None,
                    timezone: None,
                    language: None,
                    last_active_at: Some(chrono::Utc::now()),
                };

                let workspace = fechatter_core::Workspace {
                    id: claims.workspace_id.into(),
                    name: "Workspace".to_string(),
                    owner_id: claims.id.into(),
                    created_at: chrono::Utc::now(),
                };

                (user, workspace)
            } else {
                return Err(AppError::Internal(
                    "Failed to decode token claims".to_string(),
                ));
            };

            let response = LoginResponse {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token.token,
                token_type: "Bearer".to_string(),
                expires_in: ACCESS_TOKEN_EXPIRATION as i64,
                user,
                workspace,
                login_time: chrono::Utc::now(),
            };

            // Send analytics event for user login using unified analytics publisher
            if let Some(analytics_publisher) = state.analytics_publisher() {
                use crate::services::infrastructure::event::AnalyticsTracking;

                if let Err(e) = analytics_publisher
                    .track_user_login(
                        response.user.id.to_string(),
                        response.user.email.clone(),
                        "password".to_string(),
                    )
                    .await
                {
                    tracing::warn!("Failed to track login analytics event: {}", e);
                }
            }

            let api_response = ApiResponse::success(response, request_id)
                .with_duration(start_time.elapsed().as_millis() as u64);

            Ok((StatusCode::OK, response_headers, Json(api_response)).into_response())
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                crate::dtos::core::ApiError {
                    code: "INVALID_CREDENTIALS".to_string(),
                    message: "Invalid email or password".to_string(),
                    details: None,
                    field: None,
                    stack: vec![],
                    suggestion: Some("Please check your credentials and try again".to_string()),
                    help_url: Some("/docs/auth/login".to_string()),
                },
                request_id,
            )),
        )
            .into_response()),
    }
}

/// Refresh Token Handler
pub async fn refresh_token_handler(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    cookies: CookieJar,
    auth_user: Option<Extension<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
    let start_time = Instant::now();
    let request_id = extract_request_id(&headers);
    let auth_context = extract_auth_context(&headers);

    let tokens = if let Some(Extension(user)) = auth_user {
        let refresh_token = extract_refresh_token(&cookies, &headers)?;
        let auth_service =
            crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
        auth_service
            .refresh_token(&refresh_token, auth_context)
            .await?
    } else {
        let refresh_token = extract_refresh_token(&cookies, &headers)?;
        let auth_service =
            crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
        match auth_service
            .refresh_token(&refresh_token, auth_context)
            .await
        {
            Ok(tokens) => tokens,
            Err(_) => {
                let mut headers = HeaderMap::new();
                clear_refresh_token_cookie(&mut headers)?;
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    headers,
                    Json(ApiResponse::<()>::error(
                        crate::dtos::core::ApiError {
                            code: "INVALID_TOKEN".to_string(),
                            message: "Invalid or expired refresh token".to_string(),
                            details: None,
                            field: None,
                            stack: vec![],
                            suggestion: Some("Please login again".to_string()),
                            help_url: Some("/docs/auth/refresh".to_string()),
                        },
                        request_id,
                    )),
                )
                    .into_response());
            }
        }
    };

    let mut response_headers = HeaderMap::new();
    set_refresh_token_cookie(
        &mut response_headers,
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
    )?;

    let response = RefreshTokenResponse {
        access_token: tokens.access_token,
        refresh_token: Some(tokens.refresh_token.token),
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_EXPIRATION as i64,
        issued_at: chrono::Utc::now(),
    };

    let api_response = ApiResponse::success(response, request_id)
        .with_duration(start_time.elapsed().as_millis() as u64);

    Ok((StatusCode::OK, response_headers, Json(api_response)).into_response())
}

/// User Logout Handler
pub async fn logout_handler(
    Extension(state): Extension<AppState>,
    cookies: CookieJar,
    headers: HeaderMap,
    _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let start_time = Instant::now();
    let request_id = extract_request_id(&headers);

    let mut response_headers = HeaderMap::new();
    clear_refresh_token_cookie(&mut response_headers)?;

    if let Ok(refresh_token) = extract_refresh_token(&cookies, &headers) {
        let auth_service =
            crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
        auth_service
            .logout(&refresh_token)
            .await
            .map_err(AppError::from)?;
    }

    let response = LogoutResponse {
        message: "Logged out successfully".to_string(),
        success: true,
        logout_time: chrono::Utc::now(),
    };

    let api_response = ApiResponse::success(response, request_id)
        .with_duration(start_time.elapsed().as_millis() as u64);

    Ok((StatusCode::OK, response_headers, Json(api_response)).into_response())
}

/// Logout All Sessions Handler
pub async fn logout_all_handler(
    Extension(state): Extension<AppState>,
    _cookies: CookieJar,
    headers: HeaderMap,
    auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let start_time = Instant::now();
    let request_id = extract_request_id(&headers);

    let mut response_headers = HeaderMap::new();
    clear_refresh_token_cookie(&mut response_headers)?;

    let auth_service =
        crate::services::application::workers::auth::AuthUserService::from_app_state(&state);
    let _ = auth_service.logout_all(auth_user.id.into()).await;

    let response = LogoutResponse {
        message: "Logged out from all sessions successfully".to_string(),
        success: true,
        logout_time: chrono::Utc::now(),
    };

    let api_response = ApiResponse::success(response, request_id)
        .with_duration(start_time.elapsed().as_millis() as u64);

    Ok((StatusCode::OK, response_headers, Json(api_response)).into_response())
}

/// Extract refresh token from cookies or Authorization header
fn extract_refresh_token(cookies: &CookieJar, headers: &HeaderMap) -> Result<String, AppError> {
    if let Some(cookie) = cookies.get("refresh_token") {
        return Ok(cookie.value().to_string());
    }

    if let Some(auth_header) = headers.get("Authorization") {
        let auth_value = auth_header
            .to_str()
            .map_err(|_| AppError::InvalidInput("Invalid Authorization header".to_string()))?;
        if auth_value.starts_with("Bearer ") {
            return Ok(auth_value[7..].to_string());
        }
    }

    Err(AppError::InvalidInput(
        "No refresh token provided".to_string(),
    ))
}
