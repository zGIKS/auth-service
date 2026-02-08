# Shared Module Documentation

## 1. Module Overview

**Module Name:** Shared

**Purpose:**  
The Shared module provides common infrastructure, utilities, and cross-cutting concerns that are used across multiple bounded contexts in the IAM system. It contains reusable components that support system-wide requirements like auditing, resilience, security controls, and common data structures.

**Primary Responsibility:**  
Centralized management of shared infrastructure concerns including circuit breakers, rate limiting, account lockout protection, audit trails, and common REST utilities. This module ensures consistency and reusability across all bounded contexts.

**Out of Scope:**  
- Business domain logic (handled by specific bounded contexts)
- Context-specific repositories or services
- External API integrations
- Business rules or validations

## 2. Ubiquitous Language

| Term                  | Definition                                                                 | Notes |
|-----------------------|----------------------------------------------------------------------------|-------|
| Shared Kernel        | Common code and infrastructure shared across bounded contexts             | DDD pattern for shared concerns |
| Circuit Breaker      | Resilience pattern that prevents cascading failures                        | Infrastructure resilience |
| Rate Limiting        | Control of request frequency to prevent abuse                              | Security and performance |
| Account Lockout      | Temporary access restriction after failed authentication attempts         | Security protection |
| Audit Trail          | Automatic tracking of creation and modification timestamps                | Compliance and traceability |
| App State            | Centralized application configuration and shared resources                | Dependency injection container |

## 3. Domain Model Documentation

### Entities

**AuditableModel**  
- **Description:** Provides automatic audit trail functionality for entities that require tracking of creation and modification times.  
- **Invariant Rules:** Created timestamp must be set on creation, updated timestamp must be maintained on modifications.  
- **Business Examples:** Any entity requiring audit trails (identities, sessions, etc.).

## 4. Infrastructure Services Documentation

### Circuit Breaker Service

**AppCircuitBreaker**  
- **Business Capability:** Implements circuit breaker pattern to prevent cascading failures in external service calls.  
- **Inputs:** Service call requests.  
- **Outputs:** Boolean indicating if call is permitted.  
- **Business Rules:** Transitions between Closed/Open/HalfOpen states based on failure thresholds and timeouts.

### Account Lockout Service

**AccountLockoutService**  
- **Business Capability:** Prevents brute force attacks by temporarily locking accounts after failed authentication attempts.  
- **Inputs:** Identity identifier, IP address, failure thresholds.  
- **Outputs:** Lock status or failure registration confirmation.  
- **Business Rules:** Supports both global account locks and IP-specific locks with configurable thresholds.

### Rate Limiting Service

**RateLimiter**  
- **Business Capability:** Controls request frequency to prevent abuse and ensure fair resource usage.  
- **Inputs:** Client identifier, request details.  
- **Outputs:** Boolean indicating if request is allowed.  
- **Business Rules:** Implements sliding window algorithm with configurable limits.

## 5. Application Layer Documentation

(No application services - this is primarily infrastructure)

## 6. Interfaces / API Documentation

### REST Utilities

**AppState**  
- **Purpose:** Centralized container for application-wide dependencies and configuration.  
- **Components:** Database connection, Redis client, JWT secrets, timeouts, OAuth config, circuit breaker.  
- **Usage:** Injected into all HTTP handlers via Axum's FromRef trait.

**ErrorResponse**  
- **Purpose:** Standardized error response format for API consistency.  
- **Contract:** {message: string, code?: u16}.  
- **Error Scenarios:** Business errors, validation errors, infrastructure failures.

**Rate Limit Middleware**  
- **Purpose:** Applies rate limiting to all API endpoints.  
- **Behavior:** Checks request frequency, returns 429 when limits exceeded.  
- **Configuration:** Per-endpoint or global limits.

## 7. Persistence & Repositories Documentation

(No domain-specific persistence - provides infrastructure utilities)

## 8. Anti-Corruption Layer (ACL) Documentation

(No ACL needed - this module provides infrastructure to other contexts)

## 9. Context Boundaries & Integration Map

**Provides To:** All bounded contexts (Identity, Authentication, Federation, Messaging)  
**Consumes From:** None (infrastructure foundation)  
**Integration Pattern:** Direct dependency injection via AppState  
**Boundaries:** Strictly infrastructure-only, no business logic
