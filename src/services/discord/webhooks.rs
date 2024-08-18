use chrono::Local;
use dotenv::dotenv;
use rand::seq::SliceRandom;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, ExecuteWebhook};
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use serenity::model::Color;
use std::env;

pub(crate) async fn send_webhook_message(content: &str, error: Option<bool>) {
    dotenv().ok();
    let http = Http::new("");

    let webhook_uri = env::var("DISCORD_WEBHOOK_ID").expect("Expected a webhook id in the environment");
    let webhook_username = env::var("DISCORD_WEBHOOK_USERNAME").expect("Expected a webhook username in the environment");
    let webhook_avatar = env::var("DISCORD_WEBHOOK_AVATAR").expect("Expected a webhook avatar in the environment");

    let webhook = Webhook::from_url(&http, &webhook_uri).await.expect("Failed to get webhook");

    let embed_author = CreateEmbedAuthor::new("SkySync - Webhook").icon_url(&webhook_avatar);

    // Randomize the color of the embed based
    let color = if error.unwrap_or(false) {
        Color::RED
    } else {
        // Select a random color type
        let colors = vec![
            Color::BLITZ_BLUE,
            Color::DARK_PURPLE,
            Color::FOOYOO,
            Color::RED,
            Color::KERBAL,
        ];
        let random_color = colors.choose(&mut rand::thread_rng()).unwrap();
        *random_color
    };

    let embed = CreateEmbed::new().title("**New Call! - Webhook**")
        .author(embed_author)
        .description(content)
        .color(color)
        .timestamp(Local::now());
    let builder = ExecuteWebhook::new()
        .avatar_url(webhook_avatar)
        .username(webhook_username)
        .embed(embed);
    webhook.execute(&http, false, builder).await.expect("Could not execute webhook.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_webhook_message() {
        send_webhook_message("Hello, world!").await;
    }
}