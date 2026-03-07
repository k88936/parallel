# Error Handling & Logging Implementation

This document describes the robust error handling and structured logging improvements implemented in the Parallel system.

## Overview

The implementation adds comprehensive error handling, structured logging with correlation IDs, and retry logic with exponential backoff throughout the system.

## Key Components

### 1. Structured Error Types (`crates/server/src/error_codes.rs`)

- **ErrorCode Enum**: Defines standardized error codes with HTTP status mapping
  - `TASK_NOT_FOUND`, `WORKER_NOT_FOUND`, `INVALID_STATUS`, etc.
  - Each code maps to appropriate HTTP status codes
  - Provides consistent error identification across the system

### 2. API Error Response (`crates/server/src/api_error.rs`)

- **ErrorResponse Structure**: Standardized error response format
  ```json
  {
    "error": {
      "code": "TASK_NOT_FOUND",
      "message": "Task with ID xxx not found",
      "details": "Additional debug information",
      "metadata": {
        "task_id": "xxx-xxx-xxx"
      }
    },
    "correlation_id": "yyy-yyy-yyy"
  }
  ```

- **Features**:
  - User-friendly messages
  - Optional details for debugging
  - Metadata for additional context
  - Correlation ID for request tracing

### 3. Correlation ID Middleware (`crates/server/src/middleware.rs`)

- **Request ID Generation**: Automatically generates UUID for each HTTP request
- **Propagation**: Adds `x-correlation-id` header to responses
- **Tracing**: Correlation IDs are included in all logs for a request
- **Lifecycle Tracking**: Correlation IDs span the entire task lifecycle (server → worker → agent)

### 4. Retry Logic with Backoff (`crates/worker/src/utils.rs`)

- **Exponential Backoff**: Configurable retry with exponential delays
  - Initial interval: 100ms
  - Max interval: 60s
  - Multiplier: 2.0
  - Max elapsed time: 5 minutes

- **Implementation**: Uses `backoff` crate for mature retry logic
  - Replaced handwritten retry in `push_events`
  - Applied to API client operations
  - Configurable per-operation

### 5. Structured Logging

All logging statements now use structured fields:

#### Server Side
- **HTTP Requests**: `correlation_id`, `task_id`, `worker_id`, operation details
- **Task Operations**: `correlation_id`, `task_id`, `status`, error details
- **Worker Operations**: `correlation_id`, `worker_id`, operation results

#### Worker Side
- **Task Execution**: `task_id`, `repo_url`, `base_branch`, `target_branch`
- **Instruction Handling**: `worker_id`, `task_id`, instruction type
- **Git Operations**: `task_id`, `target_branch`, operation results

## Implementation Details

### Error Handling Flow

1. **Error Occurs**: Service layer returns `ServerError` or `anyhow::Error`
2. **Error Mapping**: Handlers map errors to structured `ErrorResponse`
3. **Logging**: Full context logged with correlation ID
4. **Response**: Structured JSON error response returned to client

### Correlation ID Flow

1. **HTTP Request Arrives**: Middleware generates correlation ID
2. **Request Processing**: ID included in all logs
3. **Task Creation**: Correlation ID becomes task ID
4. **Task Execution**: Worker uses task ID for correlation
5. **Sub-operations**: Nested operations tracked with same ID

### Retry Strategy

**Transient Failures** (retry with backoff):
- Network errors
- Server errors (5xx)
- Timeouts

**Permanent Failures** (no retry):
- Client errors (4xx)
- Authentication failures
- Validation errors