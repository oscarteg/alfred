use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

pub fn run(options: &[CommandDataOption]) -> String {
    // Load the id from the imdb url
    let option = options
        .get(0)
        .expect("Expected imdb option")
        .resolved
        .as_ref()
        .expect("Expected user object");

    if let CommandDataOptionValue::String(str) = option {
        format!("input: {}", str)
    } else {
        "Please provide a valid user".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("imdb")
        .description("Download a imdb movie/show")
        .create_option(|option| {
            option
                .name("id")
                .description("The user to lookup")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
