use fechatter_core::models::jwt::{TokenConfigProvider, TokenManager, UserClaims};
use fechatter_core::models::{UserId, UserStatus, WorkspaceId};
use fechatter_core::TokenService;
use chrono::Utc;

// 使用与远程服务器完全相同的密钥
struct DiagnosticAuthConfig {
    sk: String,
    pk: String,
}

impl TokenConfigProvider for DiagnosticAuthConfig {
    fn get_encoding_key_pem(&self) -> &str {
        &self.sk
    }

    fn get_decoding_key_pem(&self) -> &str {
        &self.pk
    }

    fn get_jwt_audience(&self) -> Option<&str> {
        Some("fechatter-web")
    }

    fn get_jwt_issuer(&self) -> Option<&str> {
        Some("fechatter-server")
    }

    fn get_jwt_leeway(&self) -> u64 {
        60
    }
}

fn main() {
    println!("🔍 Core JWT Diagnosis - Finding Root Cause");
    println!("==========================================");

    // 使用与远程服务器完全相同的密钥
    let config = DiagnosticAuthConfig {
        sk: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR\n-----END PRIVATE KEY-----".to_string(),
        pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=\n-----END PUBLIC KEY-----".to_string(),
    };

    println!("1. Testing TokenManager creation...");
    let token_manager = match TokenManager::new(&config) {
        Ok(tm) => {
            println!("   ✅ TokenManager created successfully");
            tm
        }
        Err(e) => {
            println!("   ❌ Failed to create TokenManager: {:?}", e);
            return;
        }
    };

    println!("2. Creating test user claims...");
    let user_claims = UserClaims {
        id: UserId::new(2),
        workspace_id: WorkspaceId::new(2),
        fullname: "Super User".to_string(),
        email: "super@test.com".to_string(),
        status: UserStatus::Active,
        created_at: Utc::now(),
    };
    println!("   ✅ User claims created: {:?}", user_claims);

    println!("3. Generating JWT token...");
    let token = match token_manager.generate_token(&user_claims) {
        Ok(t) => {
            println!("   ✅ Token generated successfully");
            println!("   📝 Token: {}...", &t[0..50]);
            t
        }
        Err(e) => {
            println!("   ❌ Failed to generate token: {:?}", e);
            return;
        }
    };

    println!("4. Analyzing token structure...");
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 {
        println!("   ✅ Token has 3 parts: header.payload.signature");
        println!("   📝 Header:    {}", parts[0]);
        println!("   📝 Payload:   {}...", &parts[1][0..20]);
        println!("   📝 Signature: {}...", &parts[2][0..20]);
    } else {
        println!("   ❌ Invalid token structure: {} parts", parts.len());
        return;
    }

    println!("5. Verifying token with same TokenManager...");
    match token_manager.verify_token(&token) {
        Ok(verified_claims) => {
            println!("   ✅ Token verification successful");
            println!("   📝 Verified user: {} ({})", verified_claims.fullname, verified_claims.email);
            println!("   📝 User ID: {:?}", verified_claims.id);
            println!("   📝 Workspace ID: {:?}", verified_claims.workspace_id);
        }
        Err(e) => {
            println!("   ❌ Token verification failed: {:?}", e);
            return;
        }
    }

    println!("6. Testing verification-only mode (notify-server scenario)...");
    let verify_config = DiagnosticAuthConfig {
        sk: "".to_string(), // Empty for verification-only mode
        pk: config.pk.clone(),
    };

    let verify_manager = match TokenManager::new(&verify_config) {
        Ok(tm) => {
            println!("   ✅ Verification-only TokenManager created");
            tm
        }
        Err(e) => {
            println!("   ❌ Failed to create verification-only TokenManager: {:?}", e);
            return;
        }
    };

    println!("7. Cross-verifying token...");
    match verify_manager.verify_token(&token) {
        Ok(verified_claims) => {
            println!("   ✅ Cross-verification successful");
            println!("   📝 Cross-verified user: {} ({})", verified_claims.fullname, verified_claims.email);
        }
        Err(e) => {
            println!("   ❌ Cross-verification failed: {:?}", e);
            return;
        }
    }

    println!("8. Testing with actual remote token format...");
    let remote_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJzdWIiOiIyIiwiZXhwIjoxNzUxMDA1MzYzLCJpYXQiOjE3NTEwMDM1NjMsImF1ZCI6ImZlY2hhdHRlci13ZWIiLCJpc3MiOiJmZWNoYXR0ZXItc2VydmVyIiwidXNlciI6eyJpZCI6Miwid29ya3NwYWNlX2lkIjoyLCJmdWxsbmFtZSI6IlN1cGVyIFVzZXIiLCJlbWFpbCI6InN1cGVyQHRlc3QuY29tIiwic3RhdHVzIjoiQWN0aXZlIiwiY3JlYXRlZF9hdCI6IjIwMjUtMDYtMTRUMDg6MDU6MDEuOTA2NDMyWiJ9fQ.1wTNF37AJKuZAwZM-yvNOiefR1UoJYk-u-C2-CFBjrup5wHawnYLyixkjOdJ_GziVbVnD8QZsFD0vALUgYXqBg";
    
    println!("   Testing remote token verification...");
    match verify_manager.verify_token(remote_token) {
        Ok(verified_claims) => {
            println!("   ✅ Remote token verification successful!");
            println!("   📝 Remote verified user: {} ({})", verified_claims.fullname, verified_claims.email);
        }
        Err(e) => {
            println!("   ❌ Remote token verification failed: {:?}", e);
            println!("   🔍 This indicates the issue is NOT in our code!");
        }
    }
    
    println!("\n🎉 Local JWT implementation is WORKING correctly!");
    println!("🔍 If remote token failed, the issue is likely:");
    println!("   1. Different key pairs being used");
    println!("   2. Configuration loading issues on remote server");
    println!("   3. Different JWT library versions in remote binary");
    println!("   4. Token corruption during transmission");
} 