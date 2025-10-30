use std::sync::Arc;

use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    api::state::AppState,
    utils::tasks::{send_daily_digest, send_weekly_roundup},
};

pub struct AgentScheduler {
    scheduler: JobScheduler,
    state: Arc<AppState>,
}

impl AgentScheduler {
    pub async fn new(state: AppState) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        let state = Arc::new(state);

        Ok(Self { scheduler, state })
    }

    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;
        tracing::info!("Scheduler started");
        Ok(())
    }

    pub async fn add_daily_digest(&self) -> Result<()> {
        let state = Arc::clone(&self.state);
        // let job = Job::new_async("1/30 * * * * *", move |_uuid, _lock| { // I left this here for testing
        let job = Job::new_async("0 0 9 * * *", move |_uuid, _lock| {
            let state = Arc::clone(&state);

            Box::pin(async move {
                tracing::info!("Running daily digest job");
                if let Err(e) = send_daily_digest(state).await {
                    tracing::error!("Daily digest failed: {}", e);
                }
            })
        })?;

        self.scheduler.add(job).await?;
        tracing::info!("Daily digest job scheduled (9 AM daily)");
        Ok(())
    }

    pub async fn add_weekly_roundup(&self) -> Result<()> {
        let state = Arc::clone(&self.state);
        let job = Job::new_async("0 0 9 * * Mon", move |_uuid, _lock| {
            let state = Arc::clone(&state);
            Box::pin(async move {
                tracing::info!("Running weekly roundup job");
                if let Err(e) = send_weekly_roundup(state).await {
                    tracing::error!("Weekly roundup failed: {}", e);
                }
            })
        })?;

        self.scheduler.add(job).await?;
        tracing::info!("Weekly roundup job scheduled (9 AM Mondays)");
        Ok(())
    }
}
