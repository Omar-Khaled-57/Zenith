use std::time::Duration;

pub const DEFAULT_BACKOFF_MS: [u64; 3] = [250, 1_000, 5_000];
pub const DEFAULT_HEALTHY_RESET: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CbState {
    Running,
    Waiting,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextAction {
    Wait(Duration),
    Failed,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    backoff_ms: Vec<u64>,
    healthy_reset: u32,
    state: CbState,
    consecutive_failures: u32,
    consecutive_healthy: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            backoff_ms: DEFAULT_BACKOFF_MS.to_vec(),
            healthy_reset: DEFAULT_HEALTHY_RESET,
            state: CbState::Running,
            consecutive_failures: 0,
            consecutive_healthy: 0,
        }
    }

    pub fn with_backoff(mut self, backoff_ms: Vec<u64>) -> Self {
        self.backoff_ms = backoff_ms;
        self
    }

    pub fn with_healthy_reset(mut self, healthy_reset: u32) -> Self {
        self.healthy_reset = healthy_reset;
        self
    }

    pub fn state(&self) -> CbState {
        self.state
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    pub fn consecutive_healthy(&self) -> u32 {
        self.consecutive_healthy
    }

    /// Reports a child failure. Returns the delay before the next respawn, or
    /// `Failed` once the backoff ladder is exhausted.
    pub fn on_failure(&mut self) -> NextAction {
        if self.consecutive_failures >= self.backoff_ms.len() as u32 {
            self.state = CbState::Failed;
            return NextAction::Failed;
        }
        let delay = Duration::from_millis(self.backoff_ms[self.consecutive_failures as usize]);
        self.consecutive_failures += 1;
        self.state = CbState::Waiting;
        NextAction::Wait(delay)
    }

    /// Marks a respawn attempt, transitioning back to running.
    pub fn on_respawn(&mut self) {
        self.state = CbState::Running;
    }

    /// Reports one healthy sample. A sustained healthy streak resets the
    /// failure counter.
    pub fn on_health(&mut self) {
        if self.state != CbState::Running {
            return;
        }
        self.consecutive_healthy += 1;
        if self.consecutive_healthy >= self.healthy_reset {
            self.consecutive_failures = 0;
            self.consecutive_healthy = 0;
        }
    }

    /// Full reset, e.g. after sleep/wake backend re-init.
    pub fn reset(&mut self) {
        self.state = CbState::Running;
        self.consecutive_failures = 0;
        self.consecutive_healthy = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_ladder_then_failed() {
        let mut cb = CircuitBreaker::new();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(1_000)));
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(5_000)));
        assert_eq!(cb.on_failure(), NextAction::Failed);
        assert_eq!(cb.state(), CbState::Failed);
        assert_eq!(cb.consecutive_failures(), 3);
    }

    #[test]
    fn respawn_keeps_ladder_progression() {
        let mut cb = CircuitBreaker::new();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
        cb.on_respawn();
        assert_eq!(cb.state(), CbState::Running);
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(1_000)));
        cb.on_respawn();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(5_000)));
        cb.on_respawn();
        assert_eq!(cb.on_failure(), NextAction::Failed);
    }

    #[test]
    fn healthy_streak_resets_failures() {
        let mut cb = CircuitBreaker::new();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
        cb.on_respawn();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(1_000)));
        cb.on_respawn();

        for _ in 0..DEFAULT_HEALTHY_RESET {
            cb.on_health();
        }
        assert_eq!(cb.consecutive_failures(), 0);

        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
    }

    #[test]
    fn healthy_streak_below_threshold_keeps_failures() {
        let mut cb = CircuitBreaker::new();
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
        cb.on_respawn();

        for _ in 0..DEFAULT_HEALTHY_RESET - 1 {
            cb.on_health();
        }
        assert_eq!(cb.consecutive_failures(), 1);
    }

    #[test]
    fn health_ignored_while_waiting() {
        let mut cb = CircuitBreaker::new();
        cb.on_failure();
        cb.on_health();
        cb.on_health();
        assert_eq!(cb.consecutive_healthy(), 0);
        assert_eq!(cb.consecutive_failures(), 1);
    }

    #[test]
    fn reset_clears_everything() {
        let mut cb = CircuitBreaker::new();
        cb.on_failure();
        cb.on_respawn();
        cb.on_failure();
        cb.on_respawn();
        cb.on_health();
        cb.reset();
        assert_eq!(cb.state(), CbState::Running);
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.consecutive_healthy(), 0);
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(250)));
    }

    #[test]
    fn custom_config() {
        let mut cb = CircuitBreaker::new()
            .with_backoff(vec![10, 20])
            .with_healthy_reset(2);
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(10)));
        assert_eq!(cb.on_failure(), NextAction::Wait(Duration::from_millis(20)));
        assert_eq!(cb.on_failure(), NextAction::Failed);

        let mut cb2 = CircuitBreaker::new()
            .with_backoff(vec![10])
            .with_healthy_reset(2);
        assert_eq!(cb2.on_failure(), NextAction::Wait(Duration::from_millis(10)));
        cb2.on_respawn();
        cb2.on_health();
        cb2.on_health();
        assert_eq!(cb2.consecutive_failures(), 0);
        assert_eq!(cb2.on_failure(), NextAction::Wait(Duration::from_millis(10)));
    }
}
