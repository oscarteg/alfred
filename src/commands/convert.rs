use anyhow::Result;
use essi_ffmpeg::FFmpeg;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use std::fs::File;
use std::io::copy;
use std::io::Cursor;

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment),
        ..
    }) = options.first()
    {
        // let resp = reqwest::blocking::get(&attachment.url).send().unwrap();

        // println!(resp);
        fetch_url(attachment.url.clone(), attachment.filename.clone())
            .await
            .unwrap();

        // Build and execute an FFmpeg command
        let mut ffmpeg = FFmpeg::new()
            .stderr(std::process::Stdio::inherit())
            .input_with_file("Goldberg.flac".into())
            .done()
            .output_as_file("output_file.aiff".into())
            .done()
            .start()
            .unwrap();

        ffmpeg.wait().unwrap();

        format!(
            "Attachment name: {}, attachment size: {}, attachment url: {}",
            attachment.filename, attachment.size, attachment.url
        )
    } else {
        "Please provide a valid attachment".to_string()
    }
}
async fn fetch_url(url: String, file_name: String) -> Result<()> {
    let response = reqwest::get(url).await.map_err(anyhow::Error::new)?;
    let mut file = File::create(file_name).map_err(anyhow::Error::new)?;
    let mut content = Cursor::new(response.bytes().await.map_err(anyhow::Error::new)?);
    copy(&mut content, &mut file).map_err(anyhow::Error::new)?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("attachmentinput")
        .description("Test command for attachment input")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Attachment, "attachment", "A file")
                .required(true),
        )
}
