use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use teloxide::{prelude::*, types::*};

use crate::utils;
use crate::Db;

pub async fn messages_handler(message: UpdateWithCx<AutoSend<Bot>, Message>, db: Db) -> Result<()> {
    let mut rng = ChaCha8Rng::from_entropy();
    if let Some(incoming_text) = message.update.text() {
        if incoming_text.contains("莎莉") {
            let sticker_set = utils::get_sticker_set(&message.requester).await?;
            if let Some(sticker) = sticker_set.stickers.choose(&mut rng) {
                message
                    .answer_sticker(InputFile::FileId(sticker.file_id.clone()))
                    .await?;
            }
        }
    } else if let Some(incoming_sticker) = message.update.sticker() {
        let key: String = rng
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        {
            let mut db = db.lock().unwrap();
            db.insert(key.clone(), incoming_sticker.clone());
        }
        message
            .answer("這個是我嗎？")
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
                inline_keyboard: vec![vec![
                    InlineKeyboardButton {
                        text: "是".into(),
                        kind: InlineKeyboardButtonKind::CallbackData(format!("{},yes", key)),
                    },
                    InlineKeyboardButton {
                        text: "否".into(),
                        kind: InlineKeyboardButtonKind::CallbackData(format!("{},no", key)),
                    },
                ]],
            }))
            .await?;
    }
    Ok(())
}

pub async fn callback_queries_handler(
    callback_query: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    db: Db,
) -> Result<()> {
    if let Some(data) = callback_query.update.data.clone() {
        let mut data = data.split(',');
        if let Some(key) = data.next() {
            if let Some(cmd) = data.next() {
                let sticker: Sticker;
                {
                    let db = db.lock().unwrap();
                    sticker = db
                        .get(key)
                        .ok_or_else(|| anyhow!("Cannot find sticker in DB"))?
                        .clone();
                }
                match cmd {
                    "yes" => utils::add_sticker(&callback_query.requester, &sticker).await?,
                    "no" => utils::remove_sticker(&callback_query.requester, &sticker).await?,
                    _ => (),
                };
            }
        }
    }
    Ok(())
}
