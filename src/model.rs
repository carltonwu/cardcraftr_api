//! Simplistic Model Layer
//! (with mock-store layer)

use crate::{ctx::Ctx, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub id: u64,
    pub cid: u64,
    pub title: String,
}

#[derive(Deserialize)]
pub struct CardToCreate {
    pub title: String,
}

#[derive(Clone)]
pub struct ModelController {
    card_store: Arc<Mutex<Vec<Option<Card>>>>,
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            card_store: Arc::default(),
        })
    }
}

impl ModelController {
    pub async fn create_card(&self, ctx: Ctx, card_tc: CardToCreate) -> Result<Card> {
        let mut store = self.card_store.lock().unwrap();

        let id = store.len() as u64;
        let card = Card {
            id,
            cid: ctx.user_id(),
            title: card_tc.title,
        };
        store.push(Some(card.clone()));

        Ok(card)
    }

    pub async fn list_cards(&self, _ctx: Ctx) -> Result<Vec<Card>> {
        let store = self.card_store.lock().unwrap();

        let card = store.iter().filter_map(|c| c.clone()).collect();

        Ok(card)
    }

    pub async fn delete_card(&self, _ctx: Ctx, id: u64) -> Result<Card> {
        let mut store = self.card_store.lock().unwrap();

        let card = store.get_mut(id as usize).and_then(|c| c.take());

        card.ok_or(Error::CardDeleteFailIdNotFound { id })

    }
}