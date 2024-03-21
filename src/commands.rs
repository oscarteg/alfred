use crate::Context;
use anyhow::Error;
use essi_ffmpeg::FFmpeg;
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::builder::CreateAttachment;
use std::fs::File;
use std::io::{copy, Cursor};

/// Show this help menu
/// Vote for something
///
/// Enter `~convert pumpkin` to vote for pumpkins
#[poise::command(prefix_command, slash_command)]
pub async fn convert(
    ctx: Context<'_>,
    #[description = "File to convert"] attachment: serenity::Attachment,
) -> Result<(), Error> {
    fetch_url(attachment.url.clone(), attachment.filename.clone())
        .await
        .unwrap();

    // Build and execute an FFmpeg command
    let mut ffmpeg = FFmpeg::new()
        .stderr(std::process::Stdio::inherit())
        .input_with_file(attachment.filename.into())
        .done()
        // .output_as_file("output_file.aiff".into())
        .output_as_file(format!("{}.aiff", attachment.filename).into::<str>())
        .done()
        .start()
        .unwrap();

    ffmpeg.wait().unwrap();

    let reply = CreateReply::default()
        .attachment(CreateAttachment::path("output_file.aiff").await.unwrap());

    ctx.send(reply).await?;

    Ok(())
}

async fn fetch_url(url: String, file_name: String) -> Result<(), Error> {
    let response = reqwest::get(url).await.map_err(anyhow::Error::new)?;
    // TODO: Use tempfile crate to create a temporary file
    let mut file = File::create(file_name).map_err(anyhow::Error::new)?;
    let mut content = Cursor::new(response.bytes().await.map_err(anyhow::Error::new)?);
    copy(&mut content, &mut file).map_err(anyhow::Error::new)?;
    Ok(())
}
