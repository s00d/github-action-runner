use std::sync::Arc;
use std::time::Duration;
use indicatif::ProgressBar;
use tokio::sync::Mutex;

pub(crate) async fn update_progress_bar(pb: Arc<Mutex<ProgressBar>>) {
    loop {
        {
            let pb = pb.lock().await;
            pb.tick();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}