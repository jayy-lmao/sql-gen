//Generated with SQLGEN
//https://github.com/jayy-lmao/sql-codegen

use sqlx::{query, query_as, PgExecutor, Result};
use super::Products;

pub struct ProductsSet;

impl ProductsSet {
    pub async fn all<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<Vec<Products>> {
        query_as::<_, Products>("SELECT * FROM Products")
            .fetch_all(executor)
            .await
    }

    pub async fn by_id<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Products> {
        query_as::<_, Products>("SELECT * FROM Products WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn by_id_optional<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Option<Products>> {
        query_as::<_, Products>("SELECT * FROM Products WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await
    }

    pub async fn all_by_categories_id<'e, E: PgExecutor<'e>>(executor: E, categories_id: i64) -> Result<Vec<Products>> {
        query_as::<_, Products>("SELECT * FROM Products WHERE category = $1")
            .bind(categories_id)
            .fetch_all(executor)
            .await
    }

    pub async fn insert<'e, E: PgExecutor<'e>>(&self, executor: E, products: Products) -> Result<Products> {
        query_as::<_, Products>("INSERT INTO Products (id, created_at, label, description, category) VALUES ($1, $2, $3, $4, $5) RETURNING *;")
            .bind(products.id)
            .bind(products.created_at)
            .bind(products.label)
            .bind(products.description)
            .bind(products.category)
            .fetch_one(executor)
            .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E, products: Products) -> Result<Products> {
        query_as::<_, Products>("UPDATE Products SET created_at = $2, label = $3, description = $4, category = $5 WHERE id = 1 RETURNING *;")
            .bind(products.id)
            .bind(products.created_at)
            .bind(products.label)
            .bind(products.description)
            .bind(products.category)
            .fetch_one(executor)
            .await
    }

    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<()> {
        query("DELETE FROM Products WHERE id = 1")
            .execute(executor)
            .await
            .map(|_| ())
    }

}
