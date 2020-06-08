use crossbeam_queue::SegQueue;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::stream::StreamExt;
use twitchchat::{events, Dispatcher, Runner, Status};

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

        let input_queue = Arc::clone(&queue);
        tokio::task::spawn(async move {
            let dispatcher = Dispatcher::new();

            let dp = dispatcher.clone();
            tokio::task::spawn(async move {
                // subscribe to 'PRIVMSG' events, this is a `Stream`
                let mut privmsgs = dp.subscribe::<events::Privmsg>();
                // 'msg' is a twitchchat::messages::Privmsg<'static> here
                while let Some(msg) = privmsgs.next().await {
                    input_queue.push(TwitchMessage {
                        channel: msg.channel.to_string(),
                        user: msg.name.to_string(),
                        body: msg.data.to_string(),
                    });
                }
            });

            let (runner, mut control) =
                Runner::new(dispatcher.clone(), twitchchat::RateLimit::default());

            let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;

            let stream = twitchchat::connect_easy_tls(&nick, &pass).await.unwrap();

            // run to completion in the background
            let done = tokio::task::spawn(runner.run(stream));

            // 'block' until connected
            let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
            //eprintln!("your irc name: {}", ready.nickname);

            // the writer is also clonable
            control.writer().join("#eva_0nline").await.unwrap();

            // this resolves when the client disconnects
            // or is forced to stop with Control::stop
            // unwrap the JoinHandle
            match done.await.unwrap() {
                // client was disconnected by the server
                Ok(Status::Eof) => {}
                // client was canceled by the user (`stop`)
                Ok(Status::Canceled) => {}
                // the client's connection timed out
                Ok(Status::Timeout) => {}
                // an error was received when trying to read or write
                Err(err) => eprintln!("error!: {}", err),
            };
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
    buffer: VecDeque<TwitchMessage>,
    n: usize,
}

impl TwitchBuffer {
    pub fn init(n: usize) -> Self {
        Self {
            twitch: Twitch::init(),
            buffer: VecDeque::with_capacity(2 * n),
            n,
        }
    }

    pub fn update(&mut self) {
        for msg in self.twitch.poll() {
            self.buffer.push_back(msg);
        }

        while self.buffer.len() > self.n {
            self.buffer.pop_front();
        }
    }

    pub fn latest(&self) -> impl DoubleEndedIterator<Item = &TwitchMessage> {
        self.buffer.iter()
    }
}
