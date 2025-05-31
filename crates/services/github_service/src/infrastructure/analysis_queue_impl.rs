use crate::domain::{AnalysisJob, JobStatus, QueueStatus};
use crate::error::{Error, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::{info, warn};

pub struct AnalysisQueueImpl {
    queue: Arc<Mutex<VecDeque<AnalysisJob>>>,
    processing: Arc<Mutex<Vec<AnalysisJob>>>,
    max_queue_size: usize,
}

impl AnalysisQueueImpl {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            processing: Arc::new(Mutex::new(Vec::new())),
            max_queue_size,
        }
    }

    pub async fn enqueue(&self, mut job: AnalysisJob) -> Result<Uuid> {
        let mut queue = self.queue.lock().await;

        if queue.len() >= self.max_queue_size {
            return Err(Error::QueueFull);
        }

        job.id = Uuid::new_v4();
        job.created_at = chrono::Utc::now();
        job.status = JobStatus::Queued;

        // Insert based on priority (higher priority first)
        let insert_position = queue
            .iter()
            .position(|existing_job| existing_job.priority < job.priority)
            .unwrap_or(queue.len());

        let job_id = job.id;
        queue.insert(insert_position, job);

        info!(
            "Enqueued analysis job {} for repository {} with priority {:?}",
            job_id, queue[insert_position].repository_id, queue[insert_position].priority
        );

        Ok(job_id)
    }

    pub async fn dequeue(&self) -> Option<AnalysisJob> {
        let mut queue = self.queue.lock().await;
        if let Some(mut job) = queue.pop_front() {
            job.status = JobStatus::Processing;

            // Move to processing list
            let mut processing = self.processing.lock().await;
            processing.push(job.clone());

            info!("Dequeued analysis job {} for processing", job.id);
            Some(job)
        } else {
            None
        }
    }

    pub async fn complete_job(&self, job_id: Uuid, success: bool) -> Result<()> {
        let mut processing = self.processing.lock().await;

        if let Some(index) = processing.iter().position(|job| job.id == job_id) {
            let mut job = processing.remove(index);
            job.status = if success {
                JobStatus::Completed
            } else {
                JobStatus::Failed
            };

            info!(
                "Analysis job {} completed with status: {:?}",
                job_id, job.status
            );

            Ok(())
        } else {
            warn!("Attempted to complete unknown job: {}", job_id);
            Err(Error::JobNotFound(job_id))
        }
    }

    pub async fn fail_job(&self, job_id: Uuid, error_message: &str) -> Result<()> {
        let mut processing = self.processing.lock().await;

        if let Some(index) = processing.iter().position(|job| job.id == job_id) {
            let mut job = processing.remove(index);
            job.status = JobStatus::Failed;

            warn!(
                "Analysis job {} failed: {}",
                job_id, error_message
            );

            Ok(())
        } else {
            warn!("Attempted to fail unknown job: {}", job_id);
            Err(Error::JobNotFound(job_id))
        }
    }

    pub async fn get_queue_status(&self) -> QueueStatus {
        let queue = self.queue.lock().await;
        let processing = self.processing.lock().await;

        QueueStatus {
            queued_jobs: queue.len(),
            processing_jobs: processing.len(),
            total_jobs: queue.len() + processing.len(),
        }
    }

    pub async fn get_job_status(&self, job_id: Uuid) -> Option<JobStatus> {
        // Check in queue
        let queue = self.queue.lock().await;
        if let Some(job) = queue.iter().find(|job| job.id == job_id) {
            return Some(job.status.clone());
        }

        // Check in processing
        let processing = self.processing.lock().await;
        if let Some(job) = processing.iter().find(|job| job.id == job_id) {
            return Some(job.status.clone());
        }

        None
    }

    pub async fn cancel_job(&self, job_id: Uuid) -> Result<()> {
        // Try to remove from queue first
        let mut queue = self.queue.lock().await;
        if let Some(index) = queue.iter().position(|job| job.id == job_id) {
            queue.remove(index);
            info!("Cancelled queued job: {}", job_id);
            return Ok(());
        }
        drop(queue);

        // Job might be processing, mark it for cancellation
        // In a real implementation, you'd need a cancellation mechanism
        warn!("Cannot cancel job {} - already processing", job_id);
        Err(Error::Internal("Cannot cancel processing job".to_string()))
    }

    pub async fn clear_completed_jobs(&self) {
        // In a real implementation, you'd persist completed jobs
        // For now, we just log that they should be cleared
        info!("Completed jobs should be persisted and cleared periodically");
    }

    pub async fn requeue_failed_jobs(&self) -> Result<usize> {
        // In a real implementation, you'd reload failed jobs and requeue them
        // This is a placeholder for that functionality
        info!("Failed jobs should be requeued based on retry policy");
        Ok(0)
    }
}

impl Default for AnalysisQueueImpl {
    fn default() -> Self {
        Self::new(1000) // Default max queue size
    }
}