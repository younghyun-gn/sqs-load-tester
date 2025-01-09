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
use tracing::{info, warn};

struct SQSMessage {
    body: String,
}

const QUEUE_URL: &str =
    "https://sqs.ap-northeast-2.amazonaws.com/547725374304/throughput-test-queue";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("SQS load tester");
    warn!("비용 발생의 위험이 있으니 주의하세요!");

    let region_provider =
        RegionProviderChain::first_try(Region::new("ap-northeast-2")).or_default_provider();

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    // Thread for logging message count and rate per second
    spawn(async move {
        let mut total: usize = 0;
        loop {
            let current_count = counter_clone.load(Ordering::Relaxed);
            total += current_count;
            info!("messages/s: {}; total: {}", current_count, total);
            counter_clone.store(0, Ordering::Relaxed);
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Thread for sending and receiving messages
    let send_client = client.clone();
    let counter_clone = Arc::clone(&counter);

    let num_threads = num_cpus::get() * 1000;
    let send_counter_clone = Arc::clone(&counter);
    let recv_counter_clone = Arc::clone(&counter);

    // Spawn threads for sending messages in parallel
    for _ in 0..num_threads {
        let send_client = client.clone();
        let send_counter_clone = Arc::clone(&send_counter_clone);

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
                    send_counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    }

    // Separate thread for receiving messages
    spawn(async move {
        loop {
            match receive(&client, &QUEUE_URL.to_string()).await {
                Ok(count) => {
                    recv_counter_clone.fetch_add(count, Ordering::Relaxed);
                }
                Err(err) => eprintln!("Failed to receive message: {:?}", err),
            }
        }
    });

    // Keep the main task alive
    loop {}
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

async fn receive(client: &Client, queue_url: &String) -> anyhow::Result<usize> {
    let rcv_message_output = client.receive_message().queue_url(queue_url).send().await?;

    // ten at a time
    Ok(rcv_message_output.messages.unwrap_or_default().len())
}
