use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    hc.do_get("/hello2/Bob").await?.print().await?;

    let req_login = hc.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "welcome"
        }));

    req_login.await?.print().await?;

    let req_create_card = hc.do_post(
        "/api/cards",
        json!({
            "title": "card aaa"
        }),
    );

    req_create_card.await?.print().await?;

    //hc.do_delete("/api/cards/1").await?.print().await?;

    hc.do_get("/api/cards").await?.print().await?;

    Ok(())
}