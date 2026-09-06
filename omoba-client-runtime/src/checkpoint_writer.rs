use std::future::Future;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use omoba_core::{
    game_proto::ClientReplicaCheckpointReport, kcp::client::ReplicaCheckpointReporter,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

use crate::ClientRuntimeError;

pub const CHECKPOINT_QUEUE_CAPACITY: usize = 512;

#[derive(Clone)]
pub struct CheckpointQueue {
    tx: mpsc::Sender<ClientReplicaCheckpointReport>,
    depth: Arc<AtomicUsize>,
    full_warned: Arc<AtomicBool>,
}

impl CheckpointQueue {
    /// 保序 enqueue。Queue 滿時只阻塞 replica runtime，不丟棄驗算資料。
    pub async fn enqueue(
        &self,
        report: ClientReplicaCheckpointReport,
    ) -> Result<(), ClientRuntimeError> {
        if self.tx.capacity() == 0 && !self.full_warned.swap(true, Ordering::Relaxed) {
            log::warn!(
                "checkpoint queue full; applying backpressure capacity={CHECKPOINT_QUEUE_CAPACITY}"
            );
        }
        let permit = self
            .tx
            .reserve()
            .await
            .map_err(|_| ClientRuntimeError::Session("checkpoint writer stopped".into()))?;
        self.depth.fetch_add(1, Ordering::Relaxed);
        permit.send(report);
        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.depth.load(Ordering::Relaxed)
    }
}

pub struct CheckpointWriter {
    failure_rx: watch::Receiver<Option<String>>,
    join: JoinHandle<Result<(), String>>,
}

impl CheckpointWriter {
    pub fn subscribe_failure(&self) -> watch::Receiver<Option<String>> {
        self.failure_rx.clone()
    }

    /// 呼叫端必須先 drop 最後一個 `CheckpointQueue`，讓 receiver 看見 EOF。
    pub async fn finish(self) -> Result<(), ClientRuntimeError> {
        self.join
            .await
            .map_err(|error| ClientRuntimeError::Session(error.to_string()))?
            .map_err(ClientRuntimeError::Session)
    }
}

pub fn spawn_checkpoint_writer(
    reporter: ReplicaCheckpointReporter,
) -> (CheckpointQueue, CheckpointWriter) {
    let (queue, rx, depth, full_warned) = checkpoint_channel(CHECKPOINT_QUEUE_CAPACITY);
    let (failure_tx, failure_rx) = watch::channel::<Option<String>>(None);
    let join = tokio::spawn(run_checkpoint_writer(
        rx,
        depth,
        full_warned,
        CHECKPOINT_QUEUE_CAPACITY,
        failure_tx,
        move |report| {
            let reporter = reporter.clone();
            async move {
                reporter
                    .report(&report)
                    .await
                    .map_err(|error| error.to_string())
            }
        },
    ));
    (queue, CheckpointWriter { failure_rx, join })
}

fn checkpoint_channel(
    capacity: usize,
) -> (
    CheckpointQueue,
    mpsc::Receiver<ClientReplicaCheckpointReport>,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
) {
    let (tx, rx) = mpsc::channel(capacity);
    let depth = Arc::new(AtomicUsize::new(0));
    let full_warned = Arc::new(AtomicBool::new(false));
    (
        CheckpointQueue {
            tx,
            depth: Arc::clone(&depth),
            full_warned: Arc::clone(&full_warned),
        },
        rx,
        depth,
        full_warned,
    )
}

async fn run_checkpoint_writer<F, Fut>(
    mut rx: mpsc::Receiver<ClientReplicaCheckpointReport>,
    depth: Arc<AtomicUsize>,
    full_warned: Arc<AtomicBool>,
    capacity: usize,
    failure_tx: watch::Sender<Option<String>>,
    mut report: F,
) -> Result<(), String>
where
    F: FnMut(ClientReplicaCheckpointReport) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    while let Some(checkpoint) = rx.recv().await {
        let result = report(checkpoint).await;
        depth.fetch_sub(1, Ordering::Relaxed);
        if depth.load(Ordering::Relaxed) < capacity {
            full_warned.store(false, Ordering::Relaxed);
        }
        if let Err(message) = result {
            let _ = failure_tx.send(Some(message.clone()));
            return Err(message);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    fn checkpoint(sequence: u64) -> ClientReplicaCheckpointReport {
        ClientReplicaCheckpointReport {
            frame_sequence: sequence,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn checkpoint_writer_preserves_fifo_order() {
        let (queue, rx, depth, warned) = checkpoint_channel(4);
        let (failure_tx, _failure_rx) = watch::channel(None);
        let seen = Arc::new(StdMutex::new(Vec::new()));
        let worker_seen = Arc::clone(&seen);
        let worker = tokio::spawn(run_checkpoint_writer(
            rx,
            depth,
            warned,
            4,
            failure_tx,
            move |value| {
                let seen = Arc::clone(&worker_seen);
                async move {
                    seen.lock().unwrap().push(value.frame_sequence);
                    Ok(())
                }
            },
        ));
        queue.enqueue(checkpoint(1)).await.unwrap();
        queue.enqueue(checkpoint(2)).await.unwrap();
        queue.enqueue(checkpoint(3)).await.unwrap();
        drop(queue);
        worker.await.unwrap().unwrap();
        assert_eq!(*seen.lock().unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn checkpoint_queue_backpressures_instead_of_dropping() {
        let (queue, mut rx, _depth, _warned) = checkpoint_channel(1);
        queue.enqueue(checkpoint(1)).await.unwrap();
        let blocked_queue = queue.clone();
        let blocked = tokio::spawn(async move { blocked_queue.enqueue(checkpoint(2)).await });
        tokio::task::yield_now().await;
        assert!(!blocked.is_finished());
        assert_eq!(rx.recv().await.unwrap().frame_sequence, 1);
        blocked.await.unwrap().unwrap();
        assert_eq!(rx.recv().await.unwrap().frame_sequence, 2);
    }

    #[tokio::test]
    async fn checkpoint_writer_failure_is_published() {
        let (queue, rx, depth, warned) = checkpoint_channel(1);
        let (failure_tx, mut failure_rx) = watch::channel(None);
        let worker = tokio::spawn(run_checkpoint_writer(
            rx,
            depth,
            warned,
            1,
            failure_tx,
            |_value| async { Err("forced checkpoint failure".to_owned()) },
        ));
        queue.enqueue(checkpoint(1)).await.unwrap();
        failure_rx.changed().await.unwrap();
        assert_eq!(
            failure_rx.borrow().as_deref(),
            Some("forced checkpoint failure")
        );
        assert!(worker.await.unwrap().is_err());
    }
}
