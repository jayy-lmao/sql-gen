
# SQL-Gen – Rust Database Codegen CLI

SQL-Gen is a lightweight tool that connects to your PostgreSQL or MySQL database (with SQLite support coming soon) and generates Rust structs that work seamlessly with the [sqlx](https://crates.io/crates/sqlx) crate. You can also choose to generate code using [db-set-macros](https://crates.io/crates/db-set-macros) if you prefer a micro-ORM style—but note that the full ORM behavior is only available in DBSet mode (and DBSet mode is currently supported only for PostgreSQL).

## Key Features

- **Automatic Code Generation:**  
  SQL-Gen inspects your database schema and generates Rust structs that map directly to your tables. These structs come with the sensible defaults (like `sqlx::FromRow`) so you can start using them immediately.

- **Choose Your Generation Style:**  
  - **sqlx Mode (default):** Generates plain Rust models compatible with sqlx.  
  - **DBSet Mode:** Uses the [db-set-macros] crate to generate models with ORM-style behavior (helper methods for queries, inserts, updates, etc.). *Note:* ORM behaviors are only available in DBSet mode, and DBSet mode is currently supported only for PostgreSQL. If you’re using MySQL, you’ll get only plain models via sqlx mode.

- **Supported Databases:**  
  Works with **PostgreSQL** and **MySQL/MariaDB**. (SQLite support will be planned next.)

- **Customizable Derives and Mappings:**  
  Add extra trait derives to your enums or models, or override the default type mappings if you have special requirements.

- **Selective Generation:**  
  Only generate code for the tables you need by using filters

## Installation

You can install SQL-Gen from [crates.io](https://crates.io/crates/sql-gen) using Cargo:

```bash
cargo install sql-gen
```

*(Want the latest updates? Install directly from GitHub with `cargo install --git https://github.com/jayy-lmao/sql-gen --branch main`.)*

## Quick Start

1. **Prepare Your Database:**  
   Make sure you have a PostgreSQL or MySQL database ready, and grab your connection URL (e.g., `postgres://user:pass@localhost:5432/mydb`).

2. **Run SQL-Gen:**  
   Generate the Rust code by running:

   ```bash
   sql-gen generate \
       --db-url postgres://user:pass@localhost:5432/mydb \
       --output src/models/
   ```

   This command connects to your database, inspects the schema, and writes the generated files into `src/models`. If the directory doesn’t exist, SQL-Gen will create it. 

3. **Review the Generated Code:**  
   For each table, SQL-Gen creates a Rust file with a struct mapping to that table. For example, a table named `users` might generate something like:

   ```rust
   // src/models/users.rs
   #[derive(Debug, sqlx::FromRow)]
   pub struct User {
       pub id: i32,
       pub name: Option<String>,
       pub email: String,
       pub created_at: chrono::DateTime<chrono::Utc>,
   }
   ```

   In **sqlx mode**, you get plain models. If you switch to **DBSet mode** (PostgreSQL only), the generated structs will have additional ORM behaviors via the db-set-macros crate:

   ```rust
   // src/models/users.rs
   #[derive(Debug, sqlx::FromRow, DbSet)]
   #[dbset(table_name = "users")]
   pub struct User {
       #[key] 
       pub id: i32,
       pub name: Option<String>,
       #[unique] 
       pub email: String,
       pub created_at: chrono::DateTime<chrono::Utc>,
   }
   ```

   In DBSet mode, the ORM-style helpers (like query builders) are generated automatically.

   ```rust
    let users = UserDbSet::many()
        .name_eq("bob".to_string()) // Can set fields to match on
        .fetch_all(pool)
        .await?;

   ```

4. **Integrate Into Your Project:**  
   Add the generated modules to your project:

   ```rust
   mod models;
   use models::users::User;
   // In DBSet mode, you will need to import the DbSet to use it:
   // use models::users_dbset::UserDbSet;
   ```

## CLI Options

SQL-Gen uses the following command-line flags:

### `--db-url <DATABASE_URL>`

**Required.** The connection string to your database (e.g., `postgres://user:pass@localhost:5432/mydb`).

### `--output <DIR>`

**Required.** 
- `-`: A dash will write to stdout 
- `models.rs`: any filename ending in `.rs` will just write all models and enums to one file.
- `models/`: any filename ending in a `/` will be written to as a module with a file-per-table and file-per-enum.


### `--mode <MODE>`  
Choose your generation mode. Options are:  

- `sqlx` (default): Generates plain models for sqlx.  
- `dbset`: Generates models with ORM behavior using [db-set-macros] (currently only supported for PostgreSQL).

### `--include-tables <LIST>`

 Generate code only for the specified comma-separated table names (e.g., `users,orders,products`).

### `--enum-derive <DERIVE TRAITS>`

Derive traits to derive for any generated enums (e.g., `Serialize,Deserialize`).
- `default` for enums is `Debug, Clone, sqlx::Type`

### `--model-derive <DERIVE TRAITS>`
Derive traits for your generated structs (e.g., `Serialize,PartialEq`).
- `default` for structs is `Debug, Clone, sqlx::FromRow`

### `--type-overrides <MAP>`

Override default SQL-to-Rust type mappings with custom values  
- `numeric=rust_decimal::Decimal,todo_status=String` will overwrite all occurences of type `numeric` to `rust_decimal::Decimal` and all occurences of the enum `todo_status` to `String`

### `--table-overrides <MAP>`

Override default SQL-to-Rust type mappings for a table column with custom values
- `status=String` will overwrite all table columns of name `status` to type `String`
- `todos.status=String` will overwrite the column `status` in table `todos` to type `String`

Run `sql-gen --help` to see the full list of options.

## Example

Imagine you have a table named `todos` in your database. Running:

```bash
sql-gen generate \
    --db-url postgres://user:pass@localhost:5432/mydb \
    --output src/models/ 
```

might generate the following code in sqlx mode:

```rust
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "todo_status")]
pub enum TodoStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "completed")]
    Completed,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TodoItem {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}
```

Switch to DBSet mode (for PostgreSQL) for some ORM helpers:

```bash
sql-gen generate \
    --db-url postgres://user:pass@localhost:5432/mydb \
    --output src/models \
    --mode dbset 
```

And the generated code might look like:

```rust
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "todo_status")]
pub enum TodoStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "completed")]
    Completed,
}

#[derive(Debug, sqlx::FromRow, DbSet)]
#[dbset(table_name = "todos")]
pub struct TodoItem {
    #[key]
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: TodoStatus,
}

```

Remember, DBSet mode is only available for PostgreSQL right now—if you’re using MySQL, you’ll need to stick with sqlx mode and plain models.

## Roadmap

- **SQLite Support:**  
  Support for SQLite is in the works.

- **DBSet support for MySQL:**  

- **Migration Generation:**  

## Contributing

We welcome feedback, bug reports, and contributions. Feel free to open an issue or submit a pull request on [GitHub](https://github.com/jayy-lmao/sql-gen).

## License

This project is available under the [MIT License](./LICENSE). 

