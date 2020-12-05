use openstreetmap_api::{Credentials, Openstreetmap};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("OPENSTREETMAP_HOST")?;
    let user = env::var("OPENSTREETMAP_USER")?;
    let password = env::var("OPENSTREETMAP_PASSWORD")?;
    let credentials = Credentials::Basic(user, password);
    let client = Openstreetmap::new(host, credentials);

    let v = client.versions().await?;

    println!("{:?}", v);

    Ok(())
}
