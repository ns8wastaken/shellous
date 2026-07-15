use std::time::{Duration, SystemTime, UNIX_EPOCH};

use calloop::channel::Sender;
use calloop::timer::{TimeoutAction, Timer};
use calloop::LoopHandle;
use chrono::{DateTime, Local};

use crate::shell::event::{ShellEvent, ShellModule};
use crate::shell::runtime::LoopData;

#[derive(Clone, Debug)]
pub struct ClockSnapshot {
    pub time: DateTime<Local>,
}

pub struct ClockService;

impl ClockService {
    pub fn new() -> Self {
        Self
    }

    fn snapshot_now() -> ClockSnapshot {
        ClockSnapshot { time: chrono::Local::now() }
    }

    /// Duration until the next minute boundary, so the timer fires right as the
    /// displayed minute changes rather than drifting against a fixed 60s cadence.
    fn until_next_interval_exact() -> Duration {
        const INTERVAL_SECS: u64 = 60;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let elapsed_secs = now.as_secs() % INTERVAL_SECS;

        // Calculate remainder in nanoseconds
        let elapsed_total_nanos = (elapsed_secs * 1_000_000_000) + (now.subsec_nanos() as u64);
        let interval_nanos = INTERVAL_SECS * 1_000_000_000;

        Duration::from_nanos(interval_nanos - elapsed_total_nanos)
    }
}

impl ShellModule for ClockService {
    fn register(&self, handle: &LoopHandle<'_, LoopData>, tx: Sender<ShellEvent>) {
        let timer = Timer::from_duration(Self::until_next_interval_exact());
        handle
            .insert_source(timer, move |_deadline, _, _data: &mut LoopData| {
                let _ = tx.send(ShellEvent::ClockUpdated(ClockSnapshot { time: Local::now() }));
                TimeoutAction::ToDuration(Duration::from_secs(60))
            })
            .expect("insert_source clock timer");
    }

    fn initial_event(&self) -> Option<ShellEvent> {
        Some(ShellEvent::ClockUpdated(ClockService::snapshot_now()))
    }
}
