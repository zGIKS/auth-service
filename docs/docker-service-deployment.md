# Docker Service Deployment

This guide runs **only the auth service** in Docker. PostgreSQL, Redis, and SMTP are external dependencies and are not started by this setup.

If PostgreSQL/Redis run on your host machine, do not use `localhost` in `.env` for container runtime. Use:

```env
DATABASE_URL=postgres://postgres:admin@host.docker.internal:5432/auth_service
REDIS_URL=redis://host.docker.internal:6379
```

On Linux, add `--add-host=host.docker.internal:host-gateway` when running the container.

Set `APP_ENV` in `.env` based on environment:

```env
APP_ENV=dev  # enables Swagger
# APP_ENV=prod  # disables Swagger
```

## Build Image

```bash
docker build -t auth-service:latest .
```

## Run Container

```bash
docker run -d \
  --name auth-service \
  --restart unless-stopped \
  --add-host=host.docker.internal:host-gateway \
  --env-file .env \
  -p 8081:8081 \
  auth-service:latest
```

## Verify Service

```bash
docker ps --filter name=auth-service
docker logs -f auth-service
```

Swagger UI (dev only): `http://localhost:8081/swagger-ui/`

## Update Container After Code Changes

Rebuild the image and recreate the container:

```bash
# 1) Build image with latest code
docker build -t auth-service:latest .

# 2) Stop old container
docker stop auth-service

# 3) Remove old container
docker rm auth-service

# 4) Start new container
docker run -d \
  --name auth-service \
  --restart unless-stopped \
  --add-host=host.docker.internal:host-gateway \
  --env-file .env \
  -p 8081:8081 \
  auth-service:latest
```

## Common Errors

```bash
# Name conflict: container already exists
docker rm -f auth-service

# Wrong image tag typo (use latest, not lat)
docker run ... auth-service:latest
```

## Common Operations

```bash
# Stop running container
docker stop auth-service

# Start existing stopped container
docker start auth-service

# Restart service
docker restart auth-service

# Check status
docker ps -a --filter name=auth-service

# Follow logs
docker logs -f auth-service
```

## Remove Service Container and Image

```bash
docker stop auth-service || true
docker rm auth-service || true
docker rmi auth-service:latest
```
