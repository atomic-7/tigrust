use color_eyre::Result;
use tigrust::ssh::AppServer;

#[tokio::main]
async fn main() -> Result<()> {
    // color_eyre::install()?;
    // let terminal = ratatui::init();
    // result.await

    println!("running on 0.0.0.0:2222");
    let mut server = AppServer::new().await;
    server.run().await.expect("failed running server");
    Ok(())
}
