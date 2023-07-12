# SQLGen - Rust CLI Tool for PostgreSQL Database Operations

SQLGen is a command-line tool written in Rust that helps you generate Rust structs, queries, and SQL migrations based on an existing PostgreSQL database schema. It utilizes the `sqlx` and `clap` libraries to provide a user-friendly and efficient experience for working with your PostgreSQL database.

This project draws inspiration from `rsdbgen` (GitHub: [brianhv/rsdbgen](https://github.com/brianhb/rsdbgen)), and we appreciate their contribution to the Rust database tooling ecosystem. Originally this was going to be extending `rsdbgen` but at a certain point of changes it seemed to have diverged.

## Features

- Generate Rust structs and queries for PostgreSQL database tables.
- Generate SQL migrations based on changes in the structs.
- Handle nullable/optional fields in the generated code.
- Optional `--force` flag to overwrite existing files.
- Use environment variables instead of clap flags (optional).

## Installation

To use SQLGen, make sure you have Rust and Cargo installed on your system. You can install them by following the instructions at [https://www.rust-lang.org/](https://www.rust-lang.org/).

Once you have Rust and Cargo installed, you can build SQLGen by running the following command:

```shell
cargo install --git https://github.com/jayy-lmao/sql-codegen --branch main
```

Replace `your-username` with your GitHub username. This will install SQLGen globally on your system, making it available as a command-line tool.

## Usage

```
sqlgen [SUBCOMMAND] [OPTIONS]
```

### Subcommands:

- `generate` - Generate structs and queries for tables
- `migrate` - Generate SQL migrations based on struct differences

### Options:

- `generate` subcommand options:
  - `-o, --output <SQLGEN_MODEL_OUTPUT_FOLDER>` - Sets the output folder for generated structs (required)
  - `-d, --database <DATABASE_URL>` - Sets the database connection URL (required)
  - `-c, --context <SQLGEN_CONTEXT_NAME>` - The name of the context for calling functions. Defaults to DB name
  - `-t, --table <SQLGEN_TABLE>` - Specify the table name(s) (multiple values supported, use delimiter)
  - `-f, --force` - Overwrites existing files sharing names in that folder

- `migrate` subcommand options:
  - `-i, --include <SQLGEN_MODEL_FOLDER>` - Sets the folder containing existing struct files (required)
  - `-t, --table <SQLGEN_TABLE>` - Specify the table name(s) (multiple values supported, use delimiter)
  - `-o, --output <SQLGEN_MIGRATION_OUTPUT>` - Sets the output folder for migrations (required)
  - `-d, --database <DATABASE_URL>` - Sets the database connection URL (required)

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
sqlgen generate --output db --database <DATABASE_URL>
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
sqlgen generate --output db --database postgresql://postgres:password@localhost/mydatabase
```

This will generate the following Rust structs and queries:


```rust
// in db/customer.rs

#[derive(sqlx::FromRow)]
struct Customer {
    pub id: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub email: Option<String>,
}

// in db/customer_db_set.rs
use sqlx::{PgExecutor, Postgres, query_as, query, Result, Error};
use super::Customer;

pub struct CustomerSet;

impl CustomerSet {
    pub async fn all<'e, E: PgExecutor<'e>>(executor: E) -> Result<Vec<Customer>, Error> {
        query_as::<_, Categories>("SELECT * FROM customer")
            .fetch_all(executor)
            .await
    }

    pub async fn by_id<'e, E: PgExecutor<'e>>(executor: E, id: i64) -> Result<Customer> {
        query_as::<_, Customer>("SELECT * FROM customer WHERE id = $1")
            .fetch_one(executor)
            .bind(id)
            .await
    }

   pub async fn by_id_optional<'e, E: PgExecutor<'e>>(executor: E, id: i64) -> Result<Option<Customer>> {
        query_as::<_, Option<Customer>>("SELECT * FROM customer WHERE id = $1")
            .fetch_optional(executor)
            .bind(id)
            .await
    }

    // Not in this example, but any FKs will also generate "all_by_<fk_table>_<fk_id>" queries
    // pub async fn all_by_purchases_id<'e, E: PgExecutor<'e>>(executor: E, purchase_id: uuid::Uuid) -> Result<Vec<Customer>> {
    //     query_as::<_, Vec<Customer>>("SELECT * FROM customer WHERE purchase_id = $1")
    //         .fetch_all(executor)
    //         .bind(category)
    //         .await
    // }

    pub async fn insert<'e, E: PgExecutor<'e>>(&self, categories: Customer, executor: E) -> Result<Customer, Error> {
        query_as::<_, Customer>("INSERT INTO customer (id, created_at, email) VALUES ($1, $2, $3)")
            .bind(categories.id)
            .bind(categories.created_at)
            .bind(categories.email)
            .execute(executor)
            .await
    }

    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<(), Error> {
        query_as::<_, Customer>("UPDATE customer SET created_at = $2, email = $3 WHERE id = 1")
            .bind(categories.id)
            .bind(categories.created_at)
            .bind(categories.email)
            .execute(executor)
    }

    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<(), Error> {
        query("DELETE FROM customer WHERE id = 1")
            .execute(executor)
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

These queries may need modifying or changing, but they can serve as a good start. You should be able to run commands like:

```rust
let customers = PostgresContext.customer.all(&pool).await?;
```

The suggested way to add customer queries etc would be to add them somewhere like `db/customer_custom_queries.rs` so that they are not overwritten by codgen. If you `impl CustomerSet` and add functions it should extend it.

### Generate Migrations

To generate SQL migrations based on changes in the structs, use the `migrate generate` command:

```shell
sqlgen migrate generate --database <DATABASE_URL> --include <FOLDER_PATH> --output migrations
```

Replace `<DATABASE_URL>` with the URL of your PostgreSQL database, `<FOLDER_PATH>` with the folder containing the generated structs (`db` in the previous example), and `migrations` with the output folder for the SQL migrations.

**Example:**

Running SQLGen with the `migrate generate` command:

```shell
sqlgen migrate generate --database postgresql://postgres:password@localhost/mydatabase --include db --output migrations
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

Running SQLGen with the `migrate generate` command will generate the following migration:

```sql
-- Migration generated for struct: Customer
ALTER TABLE customer ADD COLUMN address TEXT;
```

For a complete list of available commands and options, you can use the `--help` flag:

```shell
sqlgen --help
```

### Environment Variables

SQLGen also supports using environment variables instead of command-line flags. To use environment variables, follow these steps:

1. Create a `.env` file in your project directory.
2. Add the desired environment variables in the format `VARIABLE_NAME=VALUE`. For example:

   ```plaintext
   DATABASE_URL=postgresql://postgres:password@localhost/mydatabase
   ```

3. When running SQLGen, omit the command-line flags related to the environment variables. SQLGen will automatically read the values from the `.env` file.

   **Example:**

   ```shell
   sqlgen generate --output db
   ```

   In this example, SQLGen will read the `DATABASE_URL` value from the `.env` file.

## Roadmap

SQLGen is under active development, and future enhancements are planned. Here are some items on the roadmap:

- Clearer input types that allow for leaving out default fields
- Tests
- Linting on output (rustfmt has proven tricky with derive macros)
- Support for comments and annotations
- Support for more data types in struct and query generation.
- Integration with other database systems (MySQL, SQLite, etc.).
- Advanced migration scenarios (renaming columns, table-level changes, etc.).
- Dry run mode for generating migration code without writing to files.

Your contributions and feedback are highly appreciated! If you encounter any issues or have suggestions