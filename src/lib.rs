use worker::*;

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    // Test 1: Traditional secret (wrangler secret put)
    let traditional_result = match env.secret("TRADITIONAL_SECRET") {
        Ok(secret) => format!("Traditional secret: {}", secret.to_string()),
        Err(e) => format!("Traditional secret error: {:?}", e),
    };

    // Test 2: Secrets Store binding (secrets_store_secrets in wrangler.toml)
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
