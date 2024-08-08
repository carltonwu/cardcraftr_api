use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Json, Router};

use crate::ctx::Ctx;
use crate::model::{ModelController, Card, CardToCreate};
use crate::{Error, Result};

pub fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/cards", post(create_card).get(list_cards))
        .route("/cards/:id", delete(delete_card))
        .with_state(mc)
}

async fn create_card(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Json(card_tc): Json<CardToCreate>,
) -> Result<Json<Card>> {
    println!("->> {:<12} - create_card", "HANDLER");

    let card = mc.create_card(ctx, card_tc).await?;

    Ok(Json(card))
}

async fn list_cards(
    State(mc): State<ModelController>,
    ctx: Ctx,
) -> Result<Json<Vec<Card>>> {
    println!("->> {:<12} - list_cards", "HANDLER");

    let cards = mc.list_cards(ctx).await?;

    Ok(Json(cards))
}

async fn delete_card(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Card>> {
    println!("->> {:<12} - list_cards", "HANDLER");

    let card = mc.delete_card(ctx, id).await?;

    Ok(Json(card))
}