//Generated with SQLGEN
//https://github.com/jayy-lmao/sql-codegen

pub mod categories;
pub use categories::Categories;
pub mod categories_db_set;
pub use categories_db_set::CategoriesSet;

pub mod products;
pub use products::Products;
pub mod products_db_set;
pub use products_db_set::ProductsSet;


pub struct PostgresContext;

impl PostgresContext {
  pub fn categories(&self) -> CategoriesSet { CategoriesSet }

  pub fn products(&self) -> ProductsSet { ProductsSet }

}