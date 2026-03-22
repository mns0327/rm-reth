use tokio::sync::mpsc;

/// Errors that can occur during command dispatch or response handling.
#[derive(Debug)]
pub enum CommanError {
    /// The command channel's receiver was dropped before the command could be delivered.
    /// This typically means the worker task has exited or panicked.
    CommandRecvChannelClosed,

    /// The response channel's receiver was dropped before the response could be delivered.
    /// This typically means the caller cancelled or timed out while waiting for a response.
    ResponseRecvChannelClosed,
}

/// A command that carries data of type `T` and can send back a response of type `R`.
/// This implements a request-response pattern over async channels.
pub struct Command<T, R> {
    data: T,
    /// One-shot-style sender: used exactly once to send the response back to the caller.
    awake_sender: mpsc::Sender<R>,
}

impl<T, R> Command<T, R>
where
    T: Sync + Send + 'static,
    R: Sync + Send + 'static,
{
    /// Creates a new `Command` wrapping `data`, and returns:
    /// - the `Command` itself (to be sent to a worker/handler)
    /// - an `mpsc::Receiver<R>` the caller uses to await the response
    pub fn new(data: T) -> (Self, mpsc::Receiver<R>) {
        // Buffer of 0 means the sender will block until the receiver reads.
        // Since we expect exactly one response, this is fine.
        let (awake_sender, awake_receiver) = mpsc::channel::<R>(1); // NOTE: use 1, not 0
        let cmd = Command { data, awake_sender };
        (cmd, awake_receiver)
    }

    /// Returns a reference to the inner command data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Sends the response back to the original caller.
    /// Consumes `self` to enforce that a response is sent at most once.
    pub async fn send_response(self, response: R) -> Result<(), CommanError> {
        self.awake_sender
            .send(response)
            .await
            .map_err(|_| CommanError::ResponseRecvChannelClosed)?;
        Ok(())
    }
}

/// Extension trait that adds `send_command` to any `mpsc::Sender<Command<T, R>>`.
/// This lets callers send a command and immediately get back a receiver to await the response —
/// without needing to manually construct `Command` themselves.
pub trait CommandSenderExt<T, R>
where
    T: Sync + Send + 'static,
    R: Sync + Send + 'static,
{
    fn send_command(
        &self,
        cmd: T,
    ) -> impl std::future::Future<Output = Result<mpsc::Receiver<R>, CommanError>> + Send;
}

impl<T, R> CommandSenderExt<T, R> for mpsc::Sender<Command<T, R>>
where
    T: Sync + Send + 'static,
    R: Sync + Send + 'static,
{
    /// Wraps `cmd` in a `Command`, sends it through the channel, and returns
    /// the `Receiver` end so the caller can `.recv()` the eventual response.
    async fn send_command(&self, cmd: T) -> Result<mpsc::Receiver<R>, CommanError> {
        let (cmd, receiver) = Command::new(cmd);
        self.send(cmd)
            .await
            .map_err(|_| CommanError::CommandRecvChannelClosed)?;
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_request_response() {
        let (tx, mut rx) = mpsc::channel::<Command<String, String>>(8);

        // Simulate a worker task that handles commands
        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                let response = format!("echo: {}", cmd.data());
                cmd.send_response(response).await.unwrap();
            }
        });

        // Send a command and await the response using the extension trait
        let mut response_rx = tx.send_command("hello".to_string()).await.unwrap();
        let response = response_rx.recv().await.unwrap();

        assert_eq!(response, "echo: hello");
    }

    /// Multiple commands sent sequentially — each gets its own independent response channel.
    #[tokio::test]
    async fn test_multiple_sequential_commands() {
        let (tx, mut rx) = mpsc::channel::<Command<u32, u32>>(8);

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                let doubled = cmd.data() * 2;
                cmd.send_response(doubled).await.unwrap();
            }
        });

        for i in 0..5u32 {
            let mut resp_rx = tx.send_command(i).await.unwrap();
            let result = resp_rx.recv().await.unwrap();
            assert_eq!(result, i * 2);
        }
    }

    /// Multiple commands in-flight concurrently — responses arrive independently.
    #[tokio::test]
    async fn test_concurrent_commands() {
        let (tx, mut rx) = mpsc::channel::<Command<u32, u32>>(8);

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                let val = *cmd.data();
                cmd.send_response(val * 10).await.unwrap();
            }
        });

        // Fire all commands before awaiting any response
        let mut receivers = vec![];
        for i in 0..5u32 {
            let resp_rx = tx.send_command(i).await.unwrap();
            receivers.push((i, resp_rx));
        }

        for (i, mut resp_rx) in receivers {
            let result = resp_rx.recv().await.unwrap();
            assert_eq!(result, i * 10);
        }
    }

    /// If the worker drops the command without responding, recv() returns None.
    #[tokio::test]
    async fn test_worker_drops_command_without_responding() {
        let (tx, mut rx) = mpsc::channel::<Command<String, String>>(8);

        tokio::spawn(async move {
            if let Some(_cmd) = rx.recv().await {
                // deliberately drop `_cmd` without calling send_response
            }
        });

        let mut resp_rx = tx.send_command("ignored".to_string()).await.unwrap();
        // The sender was dropped, so recv() should return None
        let result = resp_rx.recv().await;
        assert!(result.is_none());
    }

    /// Sending on a channel whose receiver has been dropped returns an error.
    #[tokio::test]
    async fn test_send_command_fails_when_worker_gone() {
        let (tx, rx) = mpsc::channel::<Command<String, String>>(8);
        // Drop the receiver immediately — no worker is listening
        drop(rx);

        let result = tx.send_command("orphan".to_string()).await;
        assert!(result.is_err());
    }

    /// Command::data() returns a reference to the original data.
    #[tokio::test]
    async fn test_command_data_accessor() {
        let (cmd, _rx) = Command::<_, ()>::new(42u32);
        assert_eq!(*cmd.data(), 42);
    }
}
