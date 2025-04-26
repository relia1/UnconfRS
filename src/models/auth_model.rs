use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use axum_macros::FromRef;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;
use std::error::Error;

#[derive(Deserialize)]
pub struct RegistrationRequest {
    pub fname: String,
    pub lname: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub fname: String,
    pub lname: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}


// Manually implement Debug so we don't accidentally leak the password
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("password", &"[redacted]")
            .finish()
    }
}

#[async_trait]
impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromRow)]
pub struct Permission {
    pub name: String,
}

impl From<&str> for Permission {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, FromRef)]
pub struct Backend {
    pub db_pool: sqlx::Pool<sqlx::Postgres>,
}

impl Backend {
    pub fn new(db_pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { db_pool }
    }

    pub async fn has_superuser_or_staff_perms(&self, user: &User) -> Result<bool, sqlx::Error> {
        let permissions = self.get_group_permissions(user).await?;
        Ok(permissions.contains(&Permission { name: "superuser".to_string() }) || permissions.contains(&Permission { name: "staff".to_string() }))
    }

    pub async fn register(&self, new_user: RegistrationRequest) -> Result<(), Box<dyn Error>> {
        tracing::trace!("before pw hash");
        let password_hash = bcrypt::hash(&new_user.password, bcrypt::DEFAULT_COST)?;
        let user: User = sqlx::query_as(
            "INSERT INTO users (fname, lname, email, password) VALUES ($1, $2, $3, $4) RETURNING *"
        )
            .bind(&new_user.fname)
            .bind(&new_user.lname)
            .bind(&new_user.email)
            .bind(&password_hash)
            .fetch_one(&self.db_pool)
            .await?;

        tracing::trace!("user: {:?}", &user);

        sqlx::query(
            "INSERT INTO users_groups (user_id, group_id) VALUES ($1, (SELECT id FROM groups WHERE name = 'user'))"
        )
            .bind(user.id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as(r"SELECT * FROM users WHERE email = $1")
            .bind(creds.email)
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(user) = user {
            if let Ok(is_valid) = bcrypt::verify(&creds.password, &user.password) {
                if is_valid {
                    return Ok(Some(user));
                }
            }
        }
        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as(r"SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(user)
    }
}


#[async_trait]
impl AuthzBackend for Backend {
    type Permission = Permission;

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let permissions: Vec<Self::Permission> = sqlx::query_as(
            r"
            select distinct permissions.name
            from users
            join users_groups on users.id = users_groups.user_id
            join groups_permissions on users_groups.group_id = groups_permissions.group_id
            join permissions on groups_permissions.permission_id = permissions.id
            where users.id = $1
            ",
        )
            .bind(user.id)
            .fetch_all(&self.db_pool)
            .await?;

        Ok(permissions.into_iter().collect())
    }
}
