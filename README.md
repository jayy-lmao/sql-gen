# SQL-Gen - Rust CLI Tool for PostgreSQL Database Operations Generation extending SQLX

![codegen_example](https://github.com/jayy-lmao/sql-gen/assets/32926722/5308636d-3b5c-4fad-b250-109fb272d8b2)


PR's and Issues welcome! This is still early stages of devopment, though hopefully useful to some! 

SQL-Gen is a command-line tool written in Rust that helps you generate Rust structs, queries, and SQL migrations based on an existing PostgreSQL database schema. It utilizes the `sqlx` and `clap` libraries to provide a user-friendly and efficient experience for working with your PostgreSQL database.

This project draws inspiration from `rsdbgen` (GitHub: [brianhv/rsdbgen](https://github.com/brianhv/rsdbgen)) and we appreciate their contribution to the Rust database tooling ecosystem. Originally this was going to be extending `rsdbgen` but at a certain point of changes it seemed to have diverged.

## Use Cases

- Generate Rust structs and queries for PostgreSQL database tables.
- Generate SQL migrations based on changes in the structs.
- Handle nullable/optional fields in the generated code.
- Optional `--force` flag to overwrite existing files.
- Use environment variables instead of clap flags (optional).

## Installation

To use SQL-Gen, make sure you have Rust and Cargo installed on your system. You can install them by following the instructions at [https://www.rust-lang.org/](https://www.rust-lang.org/).

Once you have Rust and Cargo installed, you can build SQL-Gen by running the following command:

```shell
cargo install sql-gen
```

Or for the latest github
```shell
cargo install --git https://github.com/jayy-lmao/sql-gen --branch main
```

## Usage

```
sql-gen [SUBCOMMAND] [OPTIONS]
```

### Subcommands:

#### `generate` - Generate structs and queries for tables in your db




https://github.com/jayy-lmao/sql-gen/assets/32926722/55f3391f-47e1-42dd-b24e-6903f96971d5



#### `migrate` - Generate SQL migrations based on struct differences for structs that have a database table matching them



https://github.com/jayy-lmao/sql-gen/assets/32926722/ea3b9739-be8f-43e5-b48d-83d130ffd1c5





### Options:

- `generate` subcommand options:
  - `-o, --models <SQLGEN_MODEL_OUTPUT_FOLDER>` - Sets the output folder for generated structs (required)
  - `-d, --database <DATABASE_URL>` - Sets the database connection URL (required). Can be set as docker to spin up a testcontainer instance for applying migrations.
  - `-c, --context <SQLGEN_CONTEXT_NAME>` - The name of the context for calling functions. Defaults to DB name
  - `-f, --force` - Overwrites existing files sharing names in that folder
  - `-m, --migrations <SQLGEN_MIGRATION_INPUT>` - Sets the input folder for migrations (only if using docker as the database)

- `migrate` subcommand options:
  - `-o, --models <SQLGEN_MODEL_FOLDER>` - Sets the folder containing existing struct files (required)
  - `-m, --migrations <SQLGEN_MIGRATION_OUTPUT>` - Sets the output folder for migrations (required)
  - `-d, --database <DATABASE_URL>` - Sets the database connection URL (required). Can be set as docker to spin up a testcontainer instance for applying migrations.

### Example .env file

Create a `.env` file in the project root directory with the following content:

```
DATABASE_URL=postgres://username:password@localhost/mydatabase
SQLGEN_MODEL_OUTPUT_FOLDER=./src/models/
SQLGEN_MODEL_FOLDER=./src/models/
SQLGEN_MIGRATION_OUTPUT=./migrations
```

Make sure to replace the values with your actual database connection URL and desired folder paths for generated structs and migrations.

### Generate Structs and Queries

To generate Rust structs and queries for a PostgreSQL database, use the `generate` command:

```shell
sql-gen generate --output db --database <DATABASE_URL>
```

Replace `<DATABASE_URL>` with the URL of your PostgreSQL database. The generated code will be saved in the `db` folder.

**Example:**

Assuming we have the following input database schema:

```sql
CREATE TABLE customer (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT NOW()
    email VARCHAR(255) UNIQUE,
);
```

Running SQLGen with the `generate` command:

```shell
sql-gen generate --output db --database postgresql://postgres:password@localhost/mydatabase
```

This will generate the following Rust structs and queries (based on primary key, foreign keys, and unique fields):


```rust
// in db/customer.rs

#[derive(sqlx::FromRow, Debug)]
struct Customer {
    pub id: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub email: Option<String>,
}

// in db/customer_db_set.rs
use sqlx::{query, query_as, PgExecutor, Result};
use super::Customer;

pub struct CustomerSet;

impl CustomerSet {
    pub async fn all<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<Vec<Customer>> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer""#)
            .fetch_all(executor)
            .await
    }

    pub async fn by_id<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Customer> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE "id" = $1"#)
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn by_id_optional<'e, E: PgExecutor<'e>>(&self, executor: E, id: i64) -> Result<Option<Customer>> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE "id" = $1"#)
            .bind(id)
            .fetch_optional(executor)
            .await
    }

    // Doesn't exist in this example, but foreign keys will functions like this, assuming customer has a fk field called category
    // pub async fn all_by_categories_id<'e, E: PgExecutor<'e>>(executor: E, categories_id: i64) -> Result<Vec<Customer>> {
    //     query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE category = $1"#)
    //         .bind(categories_id)
    //         .fetch_all(executor)
    //         .await
    // }

    pub async fn by_email<'e, E: PgExecutor<'e>>(&self, executor: E, email: String) -> Result<Customer> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE "email" = $1"#)
            .bind(email)
            .fetch_one(executor)
            .await
    }

    pub async fn many_by_email_list<'e, E: PgExecutor<'e>>(&self, executor: E, email_list: Vec<String>) -> Result<Vec<Customer>> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE "email" = ANY($1)"#)
            .bind(email_list)
            .fetch_all(executor)
            .await
    }

    pub async fn by_email_optional<'e, E: PgExecutor<'e>>(&self, executor: E, email: String) -> Result<Option<Customer>> {
        query_as::<_, Customer>(r#"SELECT * FROM "customer" WHERE "email" = $1"#)
            .bind(email)
            .fetch_optional(executor)
            .await
    }


    pub async fn insert<'e, E: PgExecutor<'e>>(&self, executor: E, products: Customer) -> Result<Customer> {
        query_as::<_, Customer>(r#"INSERT INTO "customer" ("id", "created_at", "email", "category") VALUES ($1, $2, $3, $4) RETURNING *;"#)
            .bind(products.id)
            .bind(products.created_at)
            .bind(products.email)
            .fetch_one(executor)
            .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E, products: Customer) -> Result<Customer> {
        query_as::<_, Customer>(r#"UPDATE "customer" SET "created_at" = $2, "email" = $3 WHERE "id" = 1 RETURNING *;"#)
            .bind(products.id)
            .bind(products.created_at)
            .bind(products.email)
            .fetch_one(executor)
            .await
    }

    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<()> {
        query(r#"DELETE FROM "customer" WHERE "id" = 1"#)
            .execute(executor)
            .await
            .map(|_| ())
    }

}


// in db/mod.rs
pub mod customer;
pub use customer::Customer;
pub mod customer_db_set;
pub use customer_db_set::CustomerSet;

pub struct PostgresContext;

impl PostgresContext {
  pub fn customer(&self) -> CustomerSet { CustomerSet }

}
```
The name of the context will default to the name of your database, or can be set with the '--context' flag.
These queries may need modifying or changing, but they can serve as a good start. You should be able to run commands like:

```rust
let customers = PostgresContext.customer().all(&pool).await?;
```

The suggested way to add customer queries etc would be to add them somewhere like `db/customer_custom_queries.rs` so that they are not overwritten by codgen. If you `impl CustomerSet` and add functions it should extend it.

### Generate Migrations

To generate SQL migrations based on changes in the structs, use the `migrate generate` command:

```shell
sql-gen migrate generate --database <DATABASE_URL> --include <FOLDER_PATH> --output migrations
```

Replace `<DATABASE_URL>` with the URL of your PostgreSQL database, `<FOLDER_PATH>` with the folder containing the generated structs (`db` in the previous example), and `migrations` with the output folder for the SQL migrations.

**Example:**

Running SQLGen with the `migrate generate` command:

```shell
sql-gen migrate generate --database postgresql://postgres:password@localhost/mydatabase --include db --output migrations
```

This will perform a dry run of the previous database generation, compare it with the existing structs in the `db` folder, and generate SQL migrations for any detected changes. The migrations will be saved in the `migrations` folder.

**Example Migration:**

Assuming a change is made to the `Customer` struct, adding a new field:

```rust
pub struct Customer {
    pub id: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub email: Option<String>,
    pub address: Option<String>, // New field
}
```

Running SQL-Gen with the `migrate generate` command will generate the following migration:

```sql
-- Migration generated for struct: Customer
ALTER TABLE customer ADD COLUMN address TEXT;
```

For a complete list of available commands and options, you can use the `--help` flag:

```shell
sql-gen --help
```


## Roadmap

SQL-Gen is under active development, and future enhancements are planned. Here are some items on the roadmap:

Types
- [ ] Arrays / Vectors
- [ ] Enums
- [ ] Composite types
- [ ] Decimal

Other
- Singularise/Pluralise correctly with inflection crate (e.g. your table is called customers, maybe you want the struct to still be Customer)
- A way to customise generated derives
- Clearer input types that allow for leaving out default fields
- Tests
- Linting on output (rustfmt has proven tricky with derive macros)
- Support for comments and annotations
- Support for more data types in struct and query generation.
- Integration with other database systems (MySQL, SQLite, etc.).
- Advanced migration scenarios (renaming columns, table-level changes, etc.).
- Dry run mode for generating migration code without writing to files.

Your contributions and feedback are highly appreciated! If you encounter any issues or have suggestions
