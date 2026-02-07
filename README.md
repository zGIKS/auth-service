# Auth Service (IAM)

A Domain-Driven Design (DDD) implementation of an Identity and Access Management (IAM) service built with Rust and Axum.

## Features

- **Identity Management**: User registration, email confirmation, password reset
- **Authentication**: Signin/logout, JWT token management, session handling
- **Federation**: Google OAuth integration
- **Security**: Password hashing, token validation, session invalidation
- **Messaging**: Email notifications via SMTP
- **Persistence**: PostgreSQL for identities, Redis for sessions and temporary data
- **API Documentation**: OpenAPI/Swagger UI integration

## Architecture

This service follows Domain-Driven Design principles with clear separation of concerns:

- **Domain Layer**: Core business logic, entities, value objects, domain services
- **Application Layer**: Use cases orchestration, ACL for external integrations
- **Infrastructure Layer**: Database repositories, external service integrations
- **Interfaces Layer**: REST API controllers, DTOs

## Requirements

- Rust 1.70 or higher
- PostgreSQL
- Redis
- SMTP server for email notifications

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd auth-service-main
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Set up databases (PostgreSQL and Redis)

4. Configure environment variables (see .env.example)

## Configuration

Create a `.env` file in the project root with necessary configuration:

```env
DATABASE_URL=postgres://user:password@localhost/auth_service
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

## Running

```bash
cargo run
```

The server will start at `http://localhost:3000`.

## API Documentation

Access Swagger UI at `http://localhost:3000/swagger-ui/` for interactive API documentation.

## Testing

```bash
cargo test
```

## Documentation

See [IAM Bounded Context Documentation](docs/IAM_Bounded_Context_Documentation.md) for detailed domain documentation.

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