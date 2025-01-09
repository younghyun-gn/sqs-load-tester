use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_sqs::Client;
use rand::Rng;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;
use tracing::info;

struct SQSMessage {
    body: String,
}

const QUEUE_URL: &str =
    "https://sqs.ap-northeast-2.amazonaws.com/547725374304/throughput-test-queue";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let region_provider =
        RegionProviderChain::first_try(Region::new("ap-northeast-2")).or_default_provider();

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    // Thread for logging message count and rate per second
    spawn(async move {
        loop {
            let current_count = counter_clone.load(Ordering::SeqCst);
            info!("Total messages sent: {}", current_count);
            info!("Messages per second: {}", current_count);
            counter_clone.store(0, Ordering::SeqCst);
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Thread for sending and receiving messages
    let send_client = client.clone();
    let counter_clone = Arc::clone(&counter);

    spawn(async move {
        loop {
            let size = rand::thread_rng().gen_range(128..=256);
            let random_string: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(size)
                .map(char::from)
                .collect();

            let message = SQSMessage {
                body: random_string,
            };

            if let Err(err) = send(&send_client, QUEUE_URL, &message).await {
                eprintln!("Failed to send message: {:?}", err);
            } else {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }

            // Also try to receive messages in parallel
            if let Err(err) = receive(&send_client, &QUEUE_URL.to_string()).await {
                eprintln!("Failed to receive message: {:?}", err);
            }
        }
    });

    // Keep the main task alive
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c signal");
}

async fn send(client: &Client, queue_url: &str, message: &SQSMessage) -> anyhow::Result<()> {
    let rsp = client
        .send_message()
        .queue_url(queue_url)
        .message_body(&message.body)
        // If the queue is FIFO, you need to set .message_deduplication_id
        // and message_group_id or configure the queue for ContentBasedDeduplication.
        .send()
        .await?;

    Ok(())
}

async fn receive(client: &Client, queue_url: &String) -> anyhow::Result<()> {
    let rcv_message_output = client.receive_message().queue_url(queue_url).send().await?;

    // ten at a time
    for message in rcv_message_output.messages.unwrap_or_default() {
        println!("Got the message: {:#?}", message);
    }

    Ok(())
}
