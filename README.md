# Swagger Axum API

A simple API built with Axum and Rust that provides a "Hello World" endpoint with Swagger documentation.

## Features

- GET /hello endpoint that returns "Hello World"
- Automatic documentation with Swagger UI
- Port configuration through environment variables
- Asynchronous web server with Tokio

## Requirements

- Rust 1.70 or higher
- Cargo

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd auth-service
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

## Configuration

Create a `.env` file in the project root:

```
PORT=3000
```

If not specified, the default port is 3000.

## Running

```bash
cargo run
```

The server will start at `http://localhost:<PORT>`.

## API Endpoints

### GET /hello

Returns a "Hello World" message.

**Response:**
- 200 OK: "Hello World"

## Swagger Documentation

Access the interactive documentation at: `http://localhost:<PORT>/swagger-ui`

## Dependencies

- `axum`: Web framework for Rust
- `tokio`: Asynchronous runtime
- `utoipa`: OpenAPI generation
- `utoipa-swagger-ui`: Swagger UI interface
- `dotenvy`: Environment variable loading

## Project Structure

```
.
├── Cargo.toml          # Rust project configuration
├── .env                # Environment variables
├── src/
│   └── main.rs         # Main application code
└── README.md           # This documentation
```