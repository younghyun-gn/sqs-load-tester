use aws_sdk_sqs::Client;

struct SQSMessage {
    body: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
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

async fn receive(client: &Client, queue_url: &String) -> Result<(), Error> {
    let rcv_message_output = client.receive_message().queue_url(queue_url).send().await?;

    // ten at a time
    for message in rcv_message_output.messages.unwrap_or_default() {
        println!("Got the message: {:#?}", message);
    }

    Ok(())
}
