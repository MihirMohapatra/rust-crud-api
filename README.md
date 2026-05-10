# Rust CRUD API

A simple Rust CRUD API for user data using Actix Web, Diesel, and PostgreSQL.

## Requirements

- Rust and Cargo
- PostgreSQL 14 or newer
- Diesel CLI, if you want to run the included migrations:

```sh
cargo install diesel_cli --no-default-features --features postgres
```

## PostgreSQL Setup

Create a local database and user. These commands use the default `postgres`
superuser; adjust the username if your local PostgreSQL installation is
different.

```sh
psql -U postgres
```

```sql
CREATE DATABASE my_crud_app;
CREATE USER my_crud_app_user WITH PASSWORD 'my_crud_app_password';
GRANT ALL PRIVILEGES ON DATABASE my_crud_app TO my_crud_app_user;
\q
```

Copy the example environment file and update the connection string:

```sh
cp .env.example .env
```

Example `.env`:

```env
DATABASE_URL=postgres://my_crud_app_user:my_crud_app_password@localhost/my_crud_app
HOST=127.0.0.1
PORT=8080
```

Run the database migrations:

```sh
diesel migration run
```

If you need to reset the schema during development, run:

```sh
diesel migration redo
```

## Running The App

Start the server:

```sh
cargo run
```

The API listens on `http://127.0.0.1:8080` by default.

## Running With Docker

The easiest way to run the full stack is Docker Compose. It starts PostgreSQL,
builds the Rust API image, runs Diesel migrations, and exposes the API on port
`8080`.

```sh
docker compose up --build
```

After the containers are running, check the API:

```sh
curl http://127.0.0.1:8080/health
```

Create a user:

```sh
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Ada Lovelace\",\"email\":\"ada@example.com\"}"
```

Stop the containers:

```sh
docker compose down
```

Stop the containers and delete the PostgreSQL volume:

```sh
docker compose down -v
```

To run only the application image against an existing database, build and run it
with a `DATABASE_URL`:

```sh
docker build -t rust-crud-api .
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@host.docker.internal:5432/my_crud_app \
  -e HOST=0.0.0.0 \
  -e PORT=8080 \
  rust-crud-api
```

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

Fetch the created user:

```sh
curl http://127.0.0.1:8080/users/1
```

Update a user:

```sh
curl -X PUT http://127.0.0.1:8080/users/1 \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Ada Byron\",\"email\":\"ada.byron@example.com\"}"
```

Delete a user:

```sh
curl -X DELETE http://127.0.0.1:8080/users/1
```

## Testing

Run formatting checks:

```sh
cargo fmt --check
```

Run compiler and type checks:

```sh
cargo check
```

Run the test suite:

```sh
cargo test
```

This project currently relies on compile-time checks and endpoint testing with a
real PostgreSQL database. Before running integration-style checks locally, make
sure `DATABASE_URL` points to a disposable development or test database and run:

```sh
diesel migration run
```

Useful manual smoke test:

```sh
curl http://127.0.0.1:8080/health
```

The expected response is:

```json
{"status":"ok"}
```

## GitHub Actions

The workflow in `.github/workflows/ci.yml` runs on pushes and pull requests to
`main`. It starts a PostgreSQL service, installs `libpq-dev` and Diesel CLI,
runs migrations, then checks formatting, compilation, and tests.
