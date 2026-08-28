use tokio::sync::watch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShutdownReason {
    Requested,
    ServerDisconnected,
    UnsafeSession(String),
    MatchEnded,
}

#[derive(Clone)]
pub struct ShutdownToken {
    tx: watch::Sender<Option<ShutdownReason>>,
}

impl ShutdownToken {
    pub fn new() -> (Self, watch::Receiver<Option<ShutdownReason>>) {
        let (tx, rx) = watch::channel(None);
        (Self { tx }, rx)
    }

    pub fn cancel(&self, reason: ShutdownReason) {
        let _ = self.tx.send_if_modified(|current| {
            if current.is_none() {
                *current = Some(reason);
                true
            } else {
                false
            }
        });
    }
}
