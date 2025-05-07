# Fechatter Architecture

## Core Principles

The Fechatter application follows a clean architecture pattern with separation of concerns:

1. **Core Library (`fechatter_core`)**: Contains models, DTOs, interfaces (traits), and pure algorithms without any database dependencies.
2. **Server Implementation (`fechatter_server`)**: Implements the repository interfaces defined in the core, handling all database operations.

## Component Relationships

```
+------------------------+       +------------------------+
|    fechatter_core      |       |   fechatter_server     |
|                        |       |                        |
|  +----------------+    |       |  +----------------+    |
|  |     Models     |<---|-------|->| Model Impl     |    |
|  +----------------+    |       |  +----------------+    |
|                        |       |                        |
|  +----------------+    |       |  +----------------+    |
|  |  Repositories  |<---|-------|->| Repo Impl      |    |
|  | (Traits/Interf)|    |       |  | (DB Operations)|    |
|  +----------------+    |       |  +----------------+    |
|                        |       |                        |
|  +----------------+    |       |  +----------------+    |
|  | Business Logic |<---|-------|->| API Controllers|    |
|  | (Pure Algor.)  |    |       |  |                |    |
|  +----------------+    |       |  +----------------+    |
|                        |       |                        |
+------------------------+       +------------------------+
```

## Core Components

### Models

Models represent domain entities without database dependencies:

- `User` / `AuthUser`
- `Chat` / `ChatType`
- `Message`
- `Workspace`
- `ChatMember`

### Repository Traits

Repository traits define interfaces for data operations:

- `UserRepository`
- `ChatRepository`
- `MessageRepository`
- `WorkspaceRepository`
- `ChatMemberRepository`

### DTOs (Data Transfer Objects)

DTOs are used for passing data between layers:

- `CreateUser`
- `SigninUser`
- `CreateChat`
- `CreateMessage`
- `UpdateChat`

### Pure Algorithms

Pure functions that don't have side effects or database dependencies:

- `validate_chat_name`
- `process_chat_members`
- `validate_message`
- `validate_workspace_name`

## Implementation Benefits

1. **Testability**: Core business logic can be tested without database dependencies
2. **Modularity**: Components can be developed and maintained independently
3. **Flexibility**: Database implementations can be changed without affecting core logic
4. **Separation of Concerns**: Clear boundaries between different parts of the system

## Cross-Crate Trait Design

The trait design allows for different implementations of the repositories:

1. **Production Implementation**: Uses real database connections
2. **Mock Implementation**: Used for testing without real database
3. **Alternative Backends**: Could implement repositories using different databases

## Service Layer

Services orchestrate operations across multiple repositories:

- `AuthService`: Handles authentication and token management
- Future services can be added for complex business operations 