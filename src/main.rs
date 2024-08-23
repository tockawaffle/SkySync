extern crate dotenv;
mod services;

use crate::services::cloudflare::service::{dns_records, Root};
use crate::services::discord::webhooks::send_webhook_message;
use dotenv::dotenv;
use tokio::fs::{create_dir_all, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;
use tokio::time::{interval, Duration};

/// Fetches the public IP address of the current machine.
///
/// # Returns
/// A `String` containing the public IP address.
async fn get_public_ip() -> String {
    let client = reqwest::Client::new();
    let response = client.get("https://ipv4.icanhazip.com")
        .send()
        .await
        .expect("Failed to send request");

    let data = response.text().await.expect("Failed to get response");
    data
}

/// Writes a log message to a log file.
///
/// # Arguments
/// * `message` - A string slice that holds the message to be logged.
async fn write_log(message: &str) {
    let log_path = dirs::data_dir().expect("Failed to get data directory").join("SkySync");
    let log_file = log_path.join("log.txt");
    create_dir_all(&log_path).await.expect("Failed to create log directory");

    if !log_file.exists() {
        File::create(&log_file).await.expect("Failed to create log file");
    }

    let mut write_log = OpenOptions::new()
        .write(true)
        .open(&log_file)
        .await.unwrap();

    write_log.write_all(message.as_bytes()).await.expect("Failed to write to log file");
}

/// Main function that initializes the environment and starts the cron job.
///
/// This function sets up a repeating timer to check and update the public IP address
/// and DNS records at regular intervals.
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Get a handle to the Tokio runtime
    let handle = Handle::current();

    // Get the duration from the .env
    let env_duration = std::env::var("CRON_INTERVAL").expect("Expected a cron interval in the environment");
    let interval_duration = Duration::from_secs(env_duration.parse::<u64>().expect("Failed to parse cron interval"));
    let mut last_public_ip: Option<String> = None;

    // Spawn a new task that sets up a repeating timer and runs cron_init
    tokio::spawn(async move {
        let mut interval = interval(interval_duration);

        loop {
            interval.tick().await; // Wait for the next tick

            let mut msg: String = String::new();
            let start_msg = format!(
                "Running cron job at {:?}\nNext run at {:?}",
                chrono::Local::now(),
                chrono::Local::now() + interval_duration
            );
            msg.push_str(&start_msg);
            println!("{}", start_msg);

            let dns_name = std::env::var("CF_DNS_NAME").expect("Expected a DNS name in the environment");

            // Hold in memory the last public IP to compare on the next iteration
            let my_public_ip = get_public_ip().await.replace("\n", "");

            // Compare the current public IP with the last one stored
            if let Some(last_ip) = &last_public_ip {
                if my_public_ip != *last_ip {
                    // Update the last public IP
                    last_public_ip = Some(my_public_ip.clone());
                }

                // If the ip is unchanged, return the function early
                println!("Public IP has not changed: {}", my_public_ip);
                msg.push_str(&format!("\nPublic IP has not changed: {}", my_public_ip));
                write_log(&msg).await;
                continue;
            } else {
                // Update the last public IP
                last_public_ip = Some(my_public_ip.clone());
                println!("Public IP has changed to: {}", my_public_ip);
                msg.push_str(&format!("\nPublic IP has changed to: {}", my_public_ip));
            }

            let record = match dns_records(None).await {
                Ok(Root { result, .. }) => {
                    result.into_iter().find(|x| x.name == dns_name).unwrap_or_else(|| panic!("Failed to find DNS record"))
                }
                _ => {
                    panic!("Failed to fetch DNS records");
                }
            };

            if record.content == my_public_ip {
                println!("Public IP is already up to date: {}", my_public_ip);
                msg.push_str(&format!("\nPublic IP is already up to date: {}", my_public_ip));
                write_log(&msg).await;
                continue;
            }

            // Update the DNS record with the new public IP
            let update = crate::services::cloudflare::service::update_dns_records(
                &record.id,
                // Update this as needed
                crate::services::cloudflare::service::DnsType::A,
                &dns_name,
                &my_public_ip,
                1,
                false,
            ).await;

            if update.success {
                send_webhook_message(
                    &format!("O IP público do domínio {} foi atualizado com sucesso!", dns_name),
                    Option::from(false),
                ).await;
            } else {
                send_webhook_message(
                    &format!("Falha ao atualizar o IP público do domínio {}!\n\n```{}```", dns_name, update.errors[0]),
                    Option::from(true),
                ).await;
            }

            write_log(&msg).await;
        }
    });

    // Prevent main from exiting immediately
    handle.spawn_blocking(|| {
        loop {}
    }).await.unwrap();
}