# workers-rs SecretStore returns None in production

## Description

`SecretStore::get()` returns `None` in production when using `[[secrets_store_secrets]]` bindings, even though the secret exists and is correctly bound.

## Environment

- workers-rs: 0.7.4
- wrangler: 4.61.0
- Rust: stable

## Steps to Reproduce

### 1. Set up Secrets Store and secret

```bash
# List existing stores (or create one)
wrangler secrets-store store list --remote

# If you need to create a store:
wrangler secrets-store store create --name "my-store" --remote

# Create a secret in the store
wrangler secrets-store secret create <STORE_ID> \
  --name "my_secret_name" \
  --value "test-secret-value-12345" \
  --scopes "workers" \
  --remote

# Verify secret is active
wrangler secrets-store secret list <STORE_ID> --remote
```

### 2. Configure wrangler.toml

Update `store_id` in `wrangler.toml` with your actual store ID.

### 3. Deploy and add traditional secret for comparison

```bash
wrangler deploy

# Add a traditional secret (for comparison - this one works)
echo "traditional-secret-value-12345" | wrangler secret put TRADITIONAL_SECRET
```

### 4. Test

```bash
curl https://secret-store-repro.<your-subdomain>.workers.dev
```

## Expected Behavior

`secrets.get().await?` should return `Some(value)` containing the secret.

## Actual Behavior

`secrets.get().await?` returns `None`, even though:
- The secret exists in Cloudflare Secrets Store
- Wrangler deploy shows the binding correctly:
  ```
  env.MY_SECRET (store-id/secret-name)      Secrets Store Secret
  ```

## Comparison Test

This worker tests both traditional secrets and SecretStore in the same request:

```
$ curl https://secret-store-repro.cynexia.workers.dev
Traditional secret: traditional-secret-value-12345
SecretStore returned None
```

**Traditional secrets work. SecretStore does not.**

## Code

```rust
use worker::*;

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    // Test 1: Traditional secret (wrangler secret put) - WORKS
    let traditional_result = match env.secret("TRADITIONAL_SECRET") {
        Ok(secret) => format!("Traditional secret: {}", secret.to_string()),
        Err(e) => format!("Traditional secret error: {:?}", e),
    };

    // Test 2: Secrets Store binding - RETURNS NONE
    let store_result = match env.secret_store("MY_SECRET") {
        Ok(secrets) => match secrets.get().await {
            Ok(Some(value)) => format!("SecretStore value: {}", value),
            Ok(None) => "SecretStore returned None".to_string(),
            Err(e) => format!("SecretStore get error: {:?}", e),
        },
        Err(e) => format!("SecretStore binding error: {:?}", e),
    };

    Response::ok(format!("{}\n{}", traditional_result, store_result))
}
```

## wrangler.toml

```toml
[[secrets_store_secrets]]
binding = "MY_SECRET"
secret_name = "my_secret_name"
store_id = "your-store-id"
```

## Notes

- Traditional worker secrets (`wrangler secret put`) work correctly in the same worker
- Only `secrets_store_secrets` bindings are affected
- The workers-rs test suite (`test/src/secret_store.rs`) uses the same pattern
- Tests pass because they use miniflare (mock), not production Cloudflare
