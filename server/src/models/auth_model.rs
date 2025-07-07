use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use axum_macros::FromRef;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;
use std::error::Error;

#[derive(Deserialize)]
pub struct RegistrationRequest {
    pub(crate) fname: String,
    pub(crate) lname: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

impl RegistrationRequest {
    pub fn new(fname: String, lname: String, email: String, password: String) -> Self {
        Self {
            fname,
            lname,
            email,
            password,
        }
    }       
}

#[derive(Debug, Serialize)]
pub(crate) struct RegistrationResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoginResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub(crate) id: i32,
    pub(crate) fname: String,
    pub(crate) lname: String,
    pub(crate) email: String,
    #[serde(skip_serializing)]
    pub(crate) password: String,
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
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromRow, Deserialize, Serialize)]
pub struct Permission {
    pub(crate) name: String,
}

impl From<Permission> for HashSet<Permission> {
    fn from(permission: Permission) -> Self {
        HashSet::from([permission])
    }
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
    pub(crate) db_pool: sqlx::Pool<sqlx::Postgres>,
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
        let password_hash = bcrypt::hash(&new_user.password, bcrypt::DEFAULT_COST)?;
        let user: User = sqlx::query_as!(
            User,
            "INSERT INTO users (fname, lname, email, password) VALUES ($1, $2, $3, $4) RETURNING *",
            &new_user.fname,
            &new_user.lname,
            &new_user.email,
            &password_hash,
        )
            .fetch_one(&self.db_pool)
            .await?;

        sqlx::query!(
            "INSERT INTO users_groups (user_id, group_id) VALUES ($1, (SELECT id FROM groups WHERE name = 'user'))",
            user.id,
        )
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
        let user: Option<Self::User> = sqlx::query_as!(
            User,
            r"SELECT * FROM users WHERE email = $1",
            &creds.email,
        )
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(user) = user
            && let Ok(is_valid) = bcrypt::verify(&creds.password, &user.password)
            && is_valid {
            return Ok(Some(user));
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r"SELECT * FROM users WHERE id = $1",
            user_id,
        )
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
        let permissions: Vec<Self::Permission> = sqlx::query_as!(
            Permission,
            r"
            select distinct permissions.name
            from users
            join users_groups on users.id = users_groups.user_id
            join groups_permissions on users_groups.group_id = groups_permissions.group_id
            join permissions on groups_permissions.permission_id = permissions.id
            where users.id = $1
            ",
            user.id,
        )
            .fetch_all(&self.db_pool)
            .await?;

        Ok(permissions.into_iter().collect())
    }
}
