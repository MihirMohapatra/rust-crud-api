# My CRUD App

A simple Rust CRUD API for user data using Actix Web, Diesel, and PostgreSQL.

## Requirements

- Rust and Cargo
- PostgreSQL
- Diesel CLI, if you want to run the included migrations:

```sh
cargo install diesel_cli --no-default-features --features postgres
```

## Setup

1. Create a database.
2. Copy `.env.example` to `.env` and update `DATABASE_URL`.
3. Run migrations:

```sh
diesel migration run
```

4. Start the server:

```sh
cargo run
```

The API listens on `http://127.0.0.1:8080` by default.

## Endpoints

- `POST /users` creates a user.
- `GET /users/{id}` returns a user.
- `PUT /users/{id}` updates a user.
- `DELETE /users/{id}` deletes a user.

## Example

```sh
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Ada Lovelace\",\"email\":\"ada@example.com\"}"
```
