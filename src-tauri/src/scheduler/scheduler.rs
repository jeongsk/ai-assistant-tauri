//! Job scheduler - periodically checks and executes due jobs

#![allow(dead_code)]

use super::cron::CronExpression;
use super::runner::{ExecutionContext, JobExecutor, ScheduledJob};
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Configuration for the scheduler
#[derive(Clone, Debug)]
pub struct SchedulerConfig {
    /// How often to check for due jobs (in seconds)
    pub check_interval_secs: u64,
    /// Database path for system tasks
    pub db_path: String,
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60, // Check every minute
            db_path: "./app.db".to_string(),
            max_concurrent_jobs: 5,
        }
    }
}

/// Job scheduler
pub struct JobScheduler {
    config: SchedulerConfig,
    executor: Arc<JobExecutor>,
    running: Arc<RwLock<bool>>,
    jobs: Arc<RwLock<Vec<ScheduledJob>>>,
}

impl JobScheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        let exec_context = ExecutionContext {
            db_path: std::path::PathBuf::from(&config.db_path),
            agent_endpoint: None,
            timeout_secs: 300,
            agent_binary_path: None,
        };

        let executor = Arc::new(JobExecutor::new(exec_context));

        Self {
            config,
            executor,
            running: Arc::new(RwLock::new(false)),
            jobs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        if *running {
            return Err("Scheduler is already running".to_string());
        }
        *running = true;
        drop(running);

        let config = self.config.clone();
        let executor = self.executor.clone();
        let jobs = self.jobs.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(config.check_interval_secs));
            timer.tick().await; // Skip first immediate tick

            loop {
                timer.tick().await;

                // Check if still running
                {
                    let r = running_flag.read().await;
                    if !*r {
                        break;
                    }
                }

                // Clean up completed jobs
                executor.cleanup_completed().await;

                // Check for due jobs
                let due_jobs = Self::get_due_jobs(&jobs).await;

                for job in due_jobs {
                    tracing::info!("Executing due job: {}", job.name);

                    let execution_id = executor.execute_job(job.clone()).await;
                    tracing::info!("Started execution: {}", execution_id);

                    // Update next run time for the job
                    Self::update_job_next_run(&jobs, &job.id).await;
                }
            }
        });

        tracing::info!("Scheduler started with check interval: {}s", config.check_interval_secs);
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Add a job to the scheduler
    pub async fn add_job(&self, job: ScheduledJob) -> Result<(), String> {
        // Validate the cron expression
        CronExpression::parse(&job.schedule)?;

        let mut jobs = self.jobs.write().await;

        // Check for duplicate ID
        if jobs.iter().any(|j| j.id == job.id) {
            return Err(format!("Job with ID {} already exists", job.id));
        }

        jobs.push(job);
        Ok(())
    }

    /// Remove a job from the scheduler
    pub async fn remove_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(pos) = jobs.iter().position(|j| j.id == job_id) {
            jobs.remove(pos);
            true
        } else {
            false
        }
    }

    /// Update a job
    pub async fn update_job(&self, job_id: &str, mut updated_job: ScheduledJob) -> Result<(), String> {
        // Validate the cron expression
        CronExpression::parse(&updated_job.schedule)?;

        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            updated_job.id = job_id.to_string();
            *job = updated_job;
            Ok(())
        } else {
            Err(format!("Job with ID {} not found", job_id))
        }
    }

    /// Get all jobs
    pub async fn get_jobs(&self) -> Vec<ScheduledJob> {
        self.jobs.read().await.clone()
    }

    /// Get a specific job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<ScheduledJob> {
        self.jobs
            .read()
            .await
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    /// Execute a job immediately
    pub async fn execute_now(&self, job_id: &str) -> Result<String, String> {
        let job = self.get_job(job_id).await
            .ok_or_else(|| format!("Job with ID {} not found", job_id))?;

        let execution_id = self.executor.execute_job(job).await;
        Ok(execution_id)
    }

    /// Cancel a running job execution
    pub async fn cancel_execution(&self, execution_id: &str) -> bool {
        self.executor.cancel_job(execution_id).await
    }

    /// Get the number of currently running jobs
    pub async fn running_count(&self) -> usize {
        self.executor.running_count().await
    }

    /// Load jobs from a vector (e.g., from database)
    pub async fn load_jobs(&self, jobs: Vec<ScheduledJob>) -> Result<(), String> {
        let mut job_list = self.jobs.write().await;
        job_list.clear();

        for job in jobs {
            // Validate cron expression
            CronExpression::parse(&job.schedule)?;
            job_list.push(job);
        }

        Ok(())
    }

    /// Get jobs that are due to run
    async fn get_due_jobs(jobs: &Arc<RwLock<Vec<ScheduledJob>>>) -> Vec<ScheduledJob> {
        let now = Utc::now();
        let job_list = jobs.read().await;

        job_list
            .iter()
            .filter(|job| {
                // Only run enabled jobs
                if !job.enabled {
                    return false;
                }

                // Check if next_run is set and has passed
                if let Some(next_run) = job.next_run {
                    if next_run <= now {
                        return true;
                    }
                }

                false
            })
            .cloned()
            .collect()
    }

    /// Update the next run time for a job
    async fn update_job_next_run(
        jobs: &Arc<RwLock<Vec<ScheduledJob>>>,
        job_id: &str,
    ) {
        let mut job_list = jobs.write().await;
        let now = Utc::now();

        if let Some(job) = job_list.iter_mut().find(|j| j.id == job_id) {
            // Update last_run
            job.last_run = Some(now);

            // Calculate next run
            if let Ok(cron) = CronExpression::parse(&job.schedule) {
                job.next_run = cron.next_after(now);
            }
        }
    }

    /// Refresh next run times for all jobs
    pub async fn refresh_schedule(&self) {
        let mut jobs = self.jobs.write().await;
        let now = Utc::now();

        for job in jobs.iter_mut() {
            if let Ok(cron) = CronExpression::parse(&job.schedule) {
                // If next_run is not set or has passed, calculate the next one
                if job.next_run.is_none() || job.next_run.unwrap() <= now {
                    job.next_run = cron.next_after(now);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_job(id: &str, schedule: &str) -> ScheduledJob {
        ScheduledJob {
            id: id.to_string(),
            name: format!("Test Job {}", id),
            schedule: schedule.to_string(),
            job_type: crate::scheduler::runner::JobType::System,
            config: crate::scheduler::runner::JobConfig {
                target: "cleanup_old_messages".to_string(),
                params: HashMap::new(),
            },
            enabled: true,
            last_run: None,
            next_run: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = JobScheduler::new(config);

        assert!(!scheduler.is_running().await);
        assert_eq!(scheduler.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_job() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let job = create_test_job("job1", "0 * * * *");

        assert!(scheduler.add_job(job).await.is_ok());
        assert_eq!(scheduler.get_jobs().await.len(), 1);
    }

    #[tokio::test]
    async fn test_add_duplicate_job() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let job1 = create_test_job("job1", "0 * * * *");
        let job2 = create_test_job("job1", "0 0 * * *"); // Same ID

        assert!(scheduler.add_job(job1).await.is_ok());
        assert!(scheduler.add_job(job2).await.is_err());
        assert_eq!(scheduler.get_jobs().await.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_job() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let job = create_test_job("job1", "0 * * * *");

        scheduler.add_job(job).await.unwrap();
        assert!(scheduler.remove_job("job1").await);
        assert!(!scheduler.remove_job("nonexistent").await);
        assert_eq!(scheduler.get_jobs().await.len(), 0);
    }

    #[tokio::test]
    async fn test_update_job() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let job = create_test_job("job1", "0 * * * *");

        scheduler.add_job(job).await.unwrap();

        let updated_job = create_test_job("job1", "0 0 * * *");
        assert!(scheduler.update_job("job1", updated_job).await.is_ok());

        let retrieved = scheduler.get_job("job1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().schedule, "0 0 * * *");
    }

    #[tokio::test]
    async fn test_load_jobs() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let jobs = vec![
            create_test_job("job1", "0 * * * *"),
            create_test_job("job2", "0 0 * * *"),
        ];

        assert!(scheduler.load_jobs(jobs).await.is_ok());
        assert_eq!(scheduler.get_jobs().await.len(), 2);
    }

    #[tokio::test]
    async fn test_refresh_schedule() {
        let scheduler = JobScheduler::new(SchedulerConfig::default());
        let job = create_test_job("job1", "0 * * * *"); // Every hour

        scheduler.add_job(job).await.unwrap();
        scheduler.refresh_schedule().await;

        let retrieved = scheduler.get_job("job1").await;
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().next_run.is_some());
    }
}
