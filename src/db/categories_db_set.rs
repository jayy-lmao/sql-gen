//Generated with SQLGEN
//https://github.com/jayy-lmao/sql-codegen

use sqlx::{query, query_as, PgExecutor, Result};
use super::Categories;

pub struct CategoriesSet;

impl CategoriesSet {
    pub async fn all<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<Vec<Categories>> {
        query_as::<_, Categories>("SELECT * FROM Categories")
            .fetch_all(executor)
            .await
    }

    pub async fn by_id<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Categories> {
        query_as::<_, Categories>("SELECT * FROM Categories WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn by_id_optional<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Option<Categories>> {
        query_as::<_, Categories>("SELECT * FROM Categories WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await
    }


    pub async fn insert<'e, E: PgExecutor<'e>>(&self, executor: E, categories: Categories) -> Result<Categories> {
        query_as::<_, Categories>("INSERT INTO Categories (id, created_at, label) VALUES ($1, $2, $3) RETURNING *;")
            .bind(categories.id)
            .bind(categories.created_at)
            .bind(categories.label)
            .fetch_one(executor)
            .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E, categories: Categories) -> Result<Categories> {
        query_as::<_, Categories>("UPDATE Categories SET created_at = $2, label = $3 WHERE id = 1 RETURNING *;")
            .bind(categories.id)
            .bind(categories.created_at)
            .bind(categories.label)
            .fetch_one(executor)
            .await
    }

    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<()> {
        query("DELETE FROM Categories WHERE id = 1")
            .execute(executor)
            .await
            .map(|_| ())
    }

}
