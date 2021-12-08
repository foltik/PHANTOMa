use tokio::task;
use crossbeam_queue::SegQueue;
use std::collections::VecDeque;
use std::sync::Arc;
use twitchchat::{
    connector::async_std::Connector,
    messages::Commands,
    runner::{AsyncRunner, Status},
    UserConfig,
};

#[derive(Debug)]
pub struct TwitchMessage {
    pub channel: String,
    pub user: String,
    pub body: String,
}

type TwitchQueue = Arc<SegQueue<TwitchMessage>>;

pub struct Twitch {
    queue: TwitchQueue,
}

impl Twitch {
    pub fn init() -> Self {
        let queue = Arc::new(SegQueue::new());
        let tx = Arc::clone(&queue);

        task::spawn(async move {
            let connector = Connector::twitch().unwrap();
            let config = UserConfig::builder().anonymous().build().unwrap();

            let mut runner = AsyncRunner::connect(connector, &config).await.unwrap();

            // runner.join("#summit1g").await.unwrap();
            // runner.join("#timthetatman").await.unwrap();
            // runner.join("#rocketleague").await.unwrap();
            runner.join("#eva_0nline").await.unwrap();
            runner.join("#f0ltik").await.unwrap();
            runner.join("#thomaslegacy").await.unwrap();

            log::debug!("Twitch connected");

            loop {
                match runner.next_message().await.unwrap() {
                    Status::Message(Commands::Privmsg(msg)) => {
                        log::debug!("[{}] {}: {}", msg.channel(), msg.name(), msg.data());
                        tx.push(TwitchMessage {
                            channel: msg.channel().to_owned(),
                            user: msg.name().to_owned(),
                            body: msg.data().to_owned(),
                        });
                    }
                    Status::Quit | Status::Eof => break,
                    _ => {}
                }
            }
        });

        Self { queue }
    }

    pub fn poll(&self) -> Vec<TwitchMessage> {
        let mut messages = Vec::with_capacity(self.queue.len());

        while !self.queue.is_empty() {
            messages.push(self.queue.pop().unwrap());
        }

        messages
    }
}

pub struct TwitchBuffer {
    twitch: Twitch,
    buffer: VecDeque<String>,
    n: usize,
}

impl TwitchBuffer {
    pub fn new(n: usize) -> Self {
        Self {
            twitch: Twitch::init(),
            buffer: VecDeque::with_capacity(n),
            n,
        }
    }

    pub fn update(&mut self) {
        for msg in self.twitch.poll() {
            self.buffer
                .push_front(format!("[{}]: {}", msg.user, msg.body));
        }

        while self.buffer.len() > self.n {
            self.buffer.pop_back();
        }
    }

    pub fn block(&self, w: usize, h: usize) -> Vec<String> {
        self.buffer
            .iter()
            .flat_map(|s| {
                s.chars()
                    .collect::<Vec<_>>()
                    .chunks(w)
                    .map(|c| c.iter().collect::<String>())
                    .rev()
                    .collect::<Vec<_>>()
            })
            .take(h)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}
