use anyhow::{anyhow, Result};
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::types::InputSticker;
use teloxide::types::Sticker;
use teloxide::types::StickerSet;

pub async fn get_sticker_set(bot: &AutoSend<Bot>) -> Result<StickerSet> {
    let sticker_set = bot.get_sticker_set("sally_by_SallySaysBot").await?;
    Ok(sticker_set)
}

pub async fn add_sticker(bot: &AutoSend<Bot>, sticker: &Sticker) -> Result<()> {
    let input_sticker = match sticker.is_animated {
        false => InputSticker::Png(InputFile::FileId(sticker.file_id.clone())),
        true => InputSticker::Tgs(InputFile::FileId(sticker.file_id.clone())),
    };
    let emojis = sticker.emoji.clone().expect("No emojis").clone();
    bot.add_sticker_to_set(894539872, "sally_by_SallySaysBot", input_sticker, emojis)
        .await?;
    Ok(())
}

pub async fn remove_sticker(bot: &AutoSend<Bot>, sticker: &Sticker) -> Result<()> {
    let sticker_set = get_sticker_set(bot).await?;
    let file_id = sticker_set
        .stickers
        .iter()
        .find(|this_sticker| this_sticker.file_unique_id == sticker.file_unique_id)
        .ok_or_else(|| anyhow!("Cannot find sticker"))?
        .file_id
        .clone();
    bot.delete_sticker_from_set(file_id).await?;
    Ok(())
}
