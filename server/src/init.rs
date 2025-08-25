use sqlx::{Pool, Postgres};
use std::env;
use tracing::{info, warn};

/// Initialize default configurations from environment variables
///
/// This function checks for environment variables and creates default
/// unconference password and admin user if they don't already exist in the database.
pub async fn initialize_defaults(pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    initialize_unconference_password(pool).await?;
    initialize_admin_user(pool).await?;
    Ok(())
}

/// Initialize unconference password from environment variable if not exists
async fn initialize_unconference_password(pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    // Check if unconference password already exists
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conference_password")
        .fetch_one(pool)
        .await?;
    
    if existing_count == 0 {
        if let Ok(unconference_password) = env::var("UNCONFERENCE_PASSWORD") {
            info!("Setting up unconference password from environment variable");
            let hashed_password = bcrypt::hash(&unconference_password, bcrypt::DEFAULT_COST)?;
            
            sqlx::query("INSERT INTO conference_password (password) VALUES ($1)")
                .bind(&hashed_password)
                .execute(pool)
                .await?;
            
            info!("Unconference password initialized");
        } else {
            warn!("No UNCONFERENCE_PASSWORD environment variable found and no password set in database");
        }
    }
    
    Ok(())
}

/// Initialize admin user from environment variables if not exists
async fn initialize_admin_user(pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let admin_email = match env::var("ADMIN_EMAIL") {
        Ok(email) => email,
        Err(_) => return Ok(()) // Skip if no admin email provided
    };
    
    // Check if admin user already exists
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&admin_email)
        .fetch_one(pool)
        .await?;
    
    if existing_count == 0 {
        let admin_password = env::var("ADMIN_PASSWORD")?;
        let admin_name = env::var("ADMIN_NAME").unwrap_or_else(|_| "Admin User".to_string());
        
        info!("Creating admin user from environment variables");
        
        // Split name into first and last
        let name_parts: Vec<&str> = admin_name.split_whitespace().collect();
        let first_name = name_parts.first().unwrap_or(&"Admin").to_string();
        let last_name = name_parts.get(1..).map(|parts| parts.join(" ")).unwrap_or_else(|| "User".to_string());
        
        // Hash password
        let hashed_password = bcrypt::hash(&admin_password, bcrypt::DEFAULT_COST)?;
        
        // Insert user
        let user_id: i32 = sqlx::query_scalar(
            "INSERT INTO users (fname, lname, email, password) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&first_name)
        .bind(&last_name)
        .bind(&admin_email)
        .bind(&hashed_password)
        .fetch_one(pool)
        .await?;
        
        // Get admin group id
        let admin_group_id: i32 = sqlx::query_scalar("SELECT id FROM groups WHERE name = 'admin'")
            .fetch_one(pool)
            .await?;
        
        // Assign user to admin group
        sqlx::query("INSERT INTO users_groups (user_id, group_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(admin_group_id)
            .execute(pool)
            .await?;
        
        info!("Admin user created: {}", admin_email);
    }
    
    Ok(())
}