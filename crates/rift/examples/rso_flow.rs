use rift::rso::{RsoClient, authorize_url};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("RSO_CLIENT_ID").unwrap_or_else(|_| "your-client-id".to_string());

    let redirect_uri = std::env::var("RSO_REDIRECT_URI")
        .unwrap_or_else(|_| "https://app.example/callback".to_string());

    let url = authorize_url(&client_id, &redirect_uri, &["openid", "cpid"]);

    println!("1. Send the user to sign in:");
    println!("   {url}");
    println!();
    println!("2. Riot redirects back with a `code` query parameter.");
    println!("3. Exchange it for tokens:");
    println!("     let rso = RsoClient::new()?;");
    println!("     let tokens = rso");
    println!("         .token_exchange(client_id, client_secret, code, redirect_uri)");
    println!("         .await?;");

    let (Ok(client_secret), Ok(code)) = (
        std::env::var("RSO_CLIENT_SECRET"),
        std::env::var("RSO_CODE"),
    ) else {
        println!();
        println!("(set RSO_CLIENT_SECRET and RSO_CODE to perform a live exchange)");
        return Ok(());
    };

    let rso = RsoClient::new()?;

    let tokens = rso
        .token_exchange(&client_id, &client_secret, &code, &redirect_uri)
        .await?;

    println!();
    println!("access token expires in {} seconds", tokens.expires_in);

    Ok(())
}
