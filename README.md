
# SQL-Gen – Rust Database Codegen CLI

SQL-Gen is a lightweight tool that connects to your PostgreSQL or MySQL database (with SQLite support coming soon) and generates Rust structs that work seamlessly with the [sqlx] crate. You can also choose to generate code using [db-set-macros] if you prefer a micro-ORM style—but note that the full ORM behavior is only available in DBSet mode (and DBSet mode is currently supported only for PostgreSQL).

Think of SQL-Gen as a simple, focused alternative to heavy ORMs like .NET’s Entity Framework. It saves you from writing repetitive boilerplate while giving you the flexibility to write your own SQL for more complex queries.

## Key Features

- **Automatic Code Generation:**  
  SQL-Gen inspects your database schema and generates Rust structs that map directly to your tables. These structs come with the appropriate annotations (like `sqlx::FromRow`) so you can start using them immediately.

- **Choose Your Generation Style:**  
  - **sqlx Mode (default):** Generates plain Rust models compatible with sqlx.  
  - **DBSet Mode:** Uses the [db-set-macros] crate to generate models with ORM-style behavior (helper methods for queries, inserts, updates, etc.). *Note:* ORM behaviors are only available in DBSet mode, and DBSet mode is currently supported only for PostgreSQL. If you’re using MySQL, you’ll get only plain models via sqlx mode.

- **Supported Databases:**  
  Works with **PostgreSQL** and **MySQL/MariaDB**. (SQLite support is on the way.)

- **Customizable Derives and Mappings:**  
  Add extra trait derives to your enums or models, or override the default type mappings if you have special requirements.

- **Selective Generation:**  
  Only generate code for the tables you need by using filters, or rename tables on the fly with table overrides.

- **Easy to Integrate:**  
  SQL-Gen generates clean, idiomatic Rust code that you can drop right into your project. It won’t force you into a heavy ORM—you can always use the generated models with your own SQL if you prefer.

## Installation

You can install SQL-Gen from [crates.io](https://crates.io/crates/sql-gen) using Cargo:

```bash
cargo install sql-gen
```

If you only need support for one database, you can customize the installation. For example, to install with just PostgreSQL support:

```bash
cargo install sql-gen --no-default-features --features postgres
```

For MySQL:

```bash
cargo install sql-gen --no-default-features --features mysql
```

*(Want the latest updates? Install directly from GitHub with `cargo install --git https://github.com/yourusername/sql-gen --branch main`.)*

## Quick Start

1. **Prepare Your Database:**  
   Make sure you have a PostgreSQL or MySQL database ready, and grab your connection URL (e.g., `postgres://user:pass@localhost:5432/mydb`).

2. **Run SQL-Gen:**  
   Generate the Rust code by running:

   ```bash
   sql-gen generate \
       --db-url postgres://user:pass@localhost:5432/mydb \
       --output src/models \
       --mode sqlx
   ```

   This command connects to your database, inspects the schema, and writes the generated files into `src/models`. If the directory doesn’t exist, SQL-Gen will create it. If you need to overwrite existing files, add the `--overwrite-files` flag.

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
       #[key] pub id: i32,
       pub name: Option<String>,
       #[unique] pub email: String,
       pub created_at: chrono::DateTime<chrono::Utc>,
   }
   ```

   In DBSet mode, the ORM-style helpers (like query builders) are generated automatically.

4. **Integrate Into Your Project:**  
   Add the generated modules to your project:

   ```rust
   mod models;
   use models::users::User;
   // In DBSet mode, you might have additional helpers like:
   // use models::users_dbset::UserDbSet;
   ```

## CLI Options

SQL-Gen uses the following command-line flags:

- **`--db-url <DATABASE_URL>`**  
  **Required.** The connection string to your database (e.g., `postgres://user:pass@localhost:5432/mydb`).

- **`--output <DIR>`**  
  **Required.** If a `-` prints to terminal, if ends in `.rs` will be single file, if ends in `/` then it will be a folder as a rust module.

- **`--mode <MODE>`**  
  Choose your generation mode. Options are:  
  - `sqlx` (default): Generates plain models for sqlx.  
  - `dbset`: Generates models with ORM behavior using [db-set-macros] (currently only supported for PostgreSQL).

- **`--include-tables <LIST>`**  
  Generate code only for the specified comma-separated table names (e.g., `users,orders,products`).

- **`--enum-derive <TRAITS>`**  
  Extra traits to derive for any generated enums (e.g., `Serialize,Deserialize`).

- **`--model-derive <TRAITS>`**  
  Extra traits to derive for your generated structs (e.g., `Serialize,PartialEq`).

- **`--type-overrides <MAP>`**  
  Override default SQL-to-Rust type mappings with custom values (e.g., `numeric=rust_decimal::Decimal,todo_status=String`).

- **`--table-overrides <MAP>`**  
  Override default SQL-to-Rust type mappings for a specific table column with custom values (e.g., `status=String`).

Run `sql-gen --help` to see the full list of options.

## Example

Imagine you have a table named `todos` in your database. Running:

```bash
sql-gen generate \
    --db-url postgres://user:pass@localhost:5432/mydb \
    --output src/models \
    --enum-derive Debug,PartialEq \
    --table-overrides "todos=TodoItem"
```

might generate the following code in sqlx mode:

```rust
#[derive(Debug, sqlx::FromRow)]
pub struct TodoItem {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: String, // or an enum if not overridden
}
```

Switch to DBSet mode (for PostgreSQL) for some ORM helpers:

```bash
sql-gen generate \
    --db-url postgres://user:pass@localhost:5432/mydb \
    --output src/models \
    --mode dbset \
    --enum-derive Debug,PartialEq \
    --table-overrides "todos=TodoItem"
```

And the generated code might look like:

```rust
#[derive(Debug, sqlx::FromRow, DbSet)]
#[dbset(table_name = "todos")]
pub struct TodoItem {
    #[key] pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: TodoStatus,
}

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

