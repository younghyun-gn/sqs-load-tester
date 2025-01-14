# SQS Load Tester

This Rust utility is designed to test the throughput of Amazon Simple Queue Service (SQS). It does so by continuously sending and optionally receiving messages at a high rate, utilizing as many CPU threads as possible.

## How It Works

1. **Configuration:**
   - The application uses AWS SDK for Rust to interact with the SQS.
   - It initializes AWS configuration and sets up tracing for logging.

2. **Messaging:**
   - It spawns multiple threads, each responsible for sending randomly generated messages to the specified SQS queue. 
   - Another thread is dedicated to receiving messages from the queue.
   - A counter keeps track of the number of messages sent and received per second.

3. **Logging:**
   - It logs the messages sent per second and the total number sent, providing insights into the throughput.

## Important Note

- Using this script could incur a cost as it uses AWS resources (SQS).
- Ensure to configure proper permissions in your AWS settings to allow access from this script.

## Prerequisites

- Rust and Cargo installed.
- AWS account and configured credentials for your environment.

## Compiling and Running

To compile and execute with the release flag for optimized performance, follow these steps:

1. **Clone the repository:**

   ```bash
   git clone <repository-url>
   cd <repository-directory>
   ```

2. **Build the release version:**

   ```bash
   cargo build --release
   ```

3. **Run the application:**

   ```bash
   ./target/release/<executable-name>
   ```

Replace `<repository-url>` with your repository's URL and `<executable-name>` with the built executable's name.

Ensure that the `QUEUE_URL` constant in the script is set to your intended SQS queue URL.