# Merchandise Inventory Manager

This project is an inventory management system built using Rust. It utilizes various libraries and frameworks to provide a robust and efficient solution for managing merchandise inventory.

## Project Structure

```
merchandise-inventory-manager
├── src
│   ├── main.rs          # Entry point of the application
│   ├── handlers         # Contains request handlers for the application
│   ├── models           # Defines data models used in the application
│   ├── views            # Contains view templates for rendering responses
│   ├── config           # Configuration settings for the application
│   └── utils            # Utility functions and helpers
├── Cargo.toml           # Cargo configuration file with dependencies
└── README.md            # Project documentation
```

## Dependencies

The project includes the following dependencies:

- **Axum**: A web framework for building APIs.
- **Tokio**: An asynchronous runtime for Rust.
- **SQLx**: An asynchronous SQL toolkit for Rust, with support for PostgreSQL.
- **Serde**: A framework for serializing and deserializing Rust data structures.
- **Askama**: A template rendering library for Rust.
- **tower-http**: A set of HTTP utilities for Tower.
- **dotenvy**: A library for loading environment variables from a `.env` file.
- **tracing**: A framework for instrumenting Rust programs to collect structured, contextual, and async-aware diagnostics.
- **tracing-subscriber**: A subscriber for the `tracing` framework that provides logging capabilities.
- **uuid**: A library for generating and working with UUIDs.

## Getting Started

1. Clone the repository:
   ```
   git clone <repository-url>
   cd merchandise-inventory-manager
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the application:
   ```
   cargo run
   ```

## License

This project is licensed under the MIT License. See the LICENSE file for more details.

## Altering the Database
```
sqlite3 inventory.db
```

Create a table:
```
CREATE TABLE inventory (
id INTEGER PRIMARY KEY NOT NULL,
name TEXT NOT NULL,
price REAL NOT NULL DEFAULT 0.0,
image_path TEXT,
quantity INTEGER NOT NULL DEFAULT 0
);
```

Delete a table:
```
DROP TABLE inventory;
```

Add row:
```
INSERT INTO inventory (id, name, price, image_path, quantity) VALUES
('1', 'Nothing', 7.50, NULL, 5),
('2', 'Lemon', 3, 'lemon.png', 3);
```

Delete a row:
```
DELETE FROM inventory
WHERE id = '1';
```

Delete all rows:
```
DELETE FROM inventory;
```

Update a row:
```
UPDATE inventory 
SET price = 9.50
WHERE id = 4;
```

Exit:
```
.exit
```