use sqlx::PgPool;
use uuid::Uuid;

use super::model::ApiKey;

#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY,
                device_id VARCHAR(255) NOT NULL,
                api_key VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ,
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_device_id ON api_keys(device_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_api_key ON api_keys(api_key)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create(&self, api_key: &ApiKey) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, device_id, api_key, created_at, expires_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(api_key.id)
        .bind(&api_key.device_id)
        .bind(&api_key.api_key)
        .bind(api_key.created_at)
        .bind(api_key.expires_at)
        .bind(api_key.is_active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_api_key(&self, api_key: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let result = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, device_id, api_key, created_at, expires_at, is_active
            FROM api_keys
            WHERE api_key = $1
            "#,
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_device_id(&self, device_id: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let result = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, device_id, api_key, created_at, expires_at, is_active
            FROM api_keys
            WHERE device_id = $1 AND is_active = TRUE
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn deactivate_by_device_id(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = FALSE
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = FALSE
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = FALSE
            WHERE expires_at IS NOT NULL AND expires_at < NOW() AND is_active = TRUE
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
