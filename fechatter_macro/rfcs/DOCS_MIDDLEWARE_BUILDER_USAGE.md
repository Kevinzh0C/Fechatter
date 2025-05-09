# Middleware Builder — Detailed Usage Guide

This guide explains **step-by-step** how to use the `#[middleware_builder]` procedural macro that lives
in crate `fechatter_macro` to compose strongly-typed Axum middleware chains inside
`fechatter_core` (or any other crate).

> **TL;DR** – Write a tiny DSL next to an empty struct, the macro turns that into a
> compile-time safe builder whose _type parameter_ encodes which middleware layers have
> been installed so far.
>
> ```rust
> #[middleware_builder(
>     config(initial_state = NoAuth, helper_functions = true, router_ext = true),
>     state(NoAuth) -> with_auth uses "crate::middlewares::bearer_auth::verify_token_middleware" => HasAuth,
>     state(HasAuth) -> with_token_refresh uses "crate::middlewares::token_refresh::refresh_token_middleware" => AuthAndRefresh,
> )]
> pub struct GeneratedCoreBuilder;
> ```

---

## 1.  Why another builder?

Axum already offers `Router::layer(L)` but the compiler cannot tell **in which order**
(or whether at all) you added your security layers.  Previously we encoded that in
very long `impl` signatures that became unmaintainable.  This macro replaces that
boiler-plate with *type-state*: each builder value owns a phantom marker that says
_"router currently has authentication but not refresh-token logic"_ and the compiler
refuses to call `build()` until you have progressed into a "final" state.

## 2.  Conceptual model

• **States** – plain marker structs such as `NoAuth`, `HasAuth`, `AuthAndRefresh`.
They never contain data, only exist at the type level.

• **Transitions** – `fn with_auth(self) -> Builder<…, HasAuth>` etc.  Each wraps the
router in one extra `axum::middleware::from_fn_with_state` layer.

• **Builder struct** –

```rust
pub struct Generated<S, T, State = NoAuth> {
    router: axum::Router<S>,
    state:  T,
    _marker: std::marker::PhantomData<State>
}
```

* `S` –  the request-body type of your Router (same as Axum's generic)
* `T` –  your *application state* (usually `ServiceProvider`)
* `State` – one of the marker structs above, tracking what has been added.

## 3.  The mini-DSL

Syntax (comma-separated list inside the attribute):

```
config(initial_state = StartingState, helper_functions = true, router_ext = true),
state(CurrentState) -> method_name uses "path::to::middleware_fn" => NextState
```

### Configuration options

| Config parameter     | Type      | Default  | Description                                     |
|-------------------|-----------|----------|-------------------------------------------------|
| `initial_state`   | Ident     | First state | The starting state of the builder              |
| `helper_functions` | bool      | `true`   | Generate helper functions for each middleware   |
| `router_ext`      | bool      | `true`   | Generate a Router extension trait               |

### Transition rules

| Rule parts     | Meaning                                                    |
| -------------- | ---------------------------------------------------------- |
| `CurrentState` | must be a marker struct name (macro defines it for you)    |
| `method_name`  | becomes a method on the builder                            |
| `path::…::fn` | absolute or`crate::…` path to the **async fn** middleware |
| `NextState`    | marker struct after applying the method                    |

You can list rules in **any** order; duplicates are a compile-time error.

### Advanced options

You can add additional requirements to transition rules:

```
state(CurrentState) -> method_name uses "path::to::middleware_fn" => NextState requires "path::to::RequiredTrait" returns "path::to::ReturnType"
```

| Advanced option | Description                                          |
|---------------|------------------------------------------------------|
| `requires`    | Additional trait bounds for the method               |
| `returns`     | Custom return type for the method                    |

## 4.  Automatic wrapper fns

Axum's `from_fn_with_state` expects a signature like

```rust
async fn (State<AppState>, Request<Body>, Next) -> Response
```

But our in-project middleware have more exotic parameters (header map, generics
for user type, result as `Result<Response, StatusCode>` …).  The macro therefore
emits an *internal* wrapper per transition that

1. matches the middleware's shape (detected by the path's last segment),
2. converts the return type to plain `Response` if necessary,
3. adds **where-clauses** so that your application state `T` must implement the
   correct traits (`TokenVerifier`, `WithTokenManager`, `WithServiceProvider`, …).

## 5.  Required traits & helper types

| Trait / struct                             | Provided in                   | Required for                 |
| ------------------------------------------ | ----------------------------- | ---------------------------- |
| `TokenVerifier`                            | `fechatter_core::middlewares` | Bearer-auth                  |
| `WithTokenManager` + `WithServiceProvider` | ″                            | Refresh-token middleware     |
| `ActualAuthServiceProvider`                | ″                            | same as above                |
| `AuthUser`                                 | `crate::models`               | must implement`From<Claims>` |
| `TokenManager`                             | `crate::jwt`                  | implements`TokenVerifier`    |

If your own state `AppState` already wraps `ServiceProvider` you usually only need:

```rust
impl TokenVerifier for AppState { … }
impl WithTokenManager  for AppState { … }
impl WithServiceProvider for AppState { … }
```

## 6.  Step-by-step example

1. **Create the builder**

   ```rust
   // src/middlewares/custom_builder.rs
   use fechatter_macro::middleware_builder;

   #[middleware_builder(
       config(initial_state = NoAuth, helper_functions = true, router_ext = true),
       state(NoAuth) -> with_auth uses "crate::middlewares::bearer_auth::verify_token_middleware" => HasAuth,
       state(HasAuth) -> with_token_refresh uses "crate::middlewares::token_refresh::refresh_token_middleware" => AuthAndRefresh,
   )]
   pub struct GeneratedCoreBuilder;
   ```

2. **Use existing extension trait** (the macro generates one for you if router_ext = true):

   ```rust
   // The macro already generates:
   // pub trait RouterExt<S>: Sized {
   //     fn with_middlewares<T>(self, state: T) -> GeneratedCoreBuilder<S, T, NoAuth>;
   //     // ...other methods
   // }
   ```

3. **Build the chain** in your `main.rs` / router factory:

   ```rust
   let router = Router::new()
       .route("/health", get(health))
       .with_middlewares(app_state)    // => Builder<NoAuth>
       .with_auth()                    // => Builder<HasAuth>
       .with_token_refresh()           // => Builder<AuthAndRefresh>
       .build();                       // consumes builder, returns Router
   ```

   Attempting to call `with_token_refresh()` **before** `with_auth()` will not
   compile because the method is only present on the `HasAuth` impl.

## 7.  Extending with your own middleware

Suppose you implement audit logging:

```rust
pub async fn audit_middleware(State(state): State<MyState>, req: Request, next: Next) -> Response { … }
```

Add a rule:

```text
state(AuthAndRefresh) -> with_audit uses "crate::middlewares::audit::audit_middleware" => FullySecured
```

For middleware that requires additional traits, use the `requires` clause:

```text
state(HasAuth) -> with_workspace uses "crate::middlewares::workspace::workspace_middleware" => HasWorkspace requires "crate::traits::WorkspaceAware"
```

## 8.  Combining middleware with extension traits

You can create your own extension trait to provide convenience methods that apply multiple middleware layers at once:

```rust
pub trait CombinedMiddlewareExt<S>: RouterExt<S> {
    fn with_auth_refresh<T>(self, state: T) -> MiddlewareBuilder<S, T, AuthAndRefresh>
    where
        T: Clone + Send + Sync + 'static;
}

impl<S> CombinedMiddlewareExt<S> for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_auth_refresh<T>(self, state: T) -> MiddlewareBuilder<S, T, AuthAndRefresh>
    where
        T: Clone + Send + Sync + 'static,
    {
        MiddlewareBuilder::new(self, state.clone())
            .with_auth()
            .with_token_refresh()
    }
}
```

Then use it like:

```rust
let router = Router::new()
    .route("/api", get(handler))
    .with_auth_refresh(app_state)  // Apply both auth and refresh in one go
    .build();
```

## 9.  Gotchas & tips

* Use **`crate::…`** paths inside the DSL when the macro is invoked **inside** the
  same crate as the middleware.  Use fully-qualified `my_lib::…` when invoking
  across crates.
* IDEs may show red squiggles for generated identifiers until the project is
  built once.  That is normal.
* Prefer to keep each middleware's public signature narrow; the macro only needs
  to recognise the *shape*, not the internals.
* If you define your own middleware functions, ensure they return a value compatible with your Router
* To avoid conflicts between helper functions and existing implementations, set `helper_functions = false`
  and define your own helper functions with different names
* To debug macro output run:
  ```bash
  cargo rustc -p fechatter_core -- -Zunstable-options --pretty=expanded > /tmp/expanded.rs
  ```

  and open `/tmp/expanded.rs`.

## 10.  Roadmap / TODO

1. Support an explicit `build()` rule in DSL to identify final states.
2. Allow per-transition **extra generic bounds** via an optional `where { … }`
   clause in the DSL.
3. Replace heuristic detection with explicit keywords:
   ```text
   state(…) -> with_auth uses_auth "…path…" => …
   ```
4. Add support for middleware factories that don't take a state parameter
5. Improve error messages for common configuration mistakes

---

Happy hacking!  Bug reports & pull requests welcome.
