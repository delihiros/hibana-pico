use crate::choreography::protocol::{BudgetExpired, BudgetRestart, BudgetRun, BudgetSuspend};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetError {
    AlreadyRunning,
    NoActiveRun,
    NotRunning,
    NotExpired,
    NotSuspended,
    RunMismatch,
    GenerationMismatch,
    StaleGeneration,
    ZeroFuel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetState {
    Idle,
    Running(BudgetRun),
    Expired(BudgetExpired),
    Suspended(BudgetSuspend),
}

pub struct BudgetController {
    state: BudgetState,
}

impl BudgetController {
    pub const fn new() -> Self {
        Self {
            state: BudgetState::Idle,
        }
    }

    pub const fn state(&self) -> BudgetState {
        self.state
    }

    pub fn start(&mut self, run: BudgetRun) -> Result<BudgetRun, BudgetError> {
        if run.fuel() == 0 {
            return Err(BudgetError::ZeroFuel);
        }
        match self.state {
            BudgetState::Idle => {
                self.state = BudgetState::Running(run);
                Ok(run)
            }
            _ => Err(BudgetError::AlreadyRunning),
        }
    }

    pub fn admit_expiry(&mut self, expired: BudgetExpired) -> Result<BudgetExpired, BudgetError> {
        match self.state {
            BudgetState::Running(run) => {
                if run.run_id() != expired.run_id() {
                    return Err(BudgetError::RunMismatch);
                }
                if run.generation() != expired.generation() {
                    return Err(BudgetError::GenerationMismatch);
                }
                self.state = BudgetState::Expired(expired);
                Ok(expired)
            }
            BudgetState::Idle => Err(BudgetError::NoActiveRun),
            _ => Err(BudgetError::NotRunning),
        }
    }

    pub fn suspend_after_expiry(&mut self) -> Result<BudgetSuspend, BudgetError> {
        match self.state {
            BudgetState::Expired(expired) => {
                self.state = BudgetState::Suspended(expired);
                Ok(expired)
            }
            BudgetState::Idle => Err(BudgetError::NoActiveRun),
            _ => Err(BudgetError::NotExpired),
        }
    }

    pub fn restart_after_suspend(
        &mut self,
        restart: BudgetRestart,
    ) -> Result<BudgetRestart, BudgetError> {
        if restart.fuel() == 0 {
            return Err(BudgetError::ZeroFuel);
        }
        match self.state {
            BudgetState::Suspended(suspended) => {
                if restart.run_id() != suspended.run_id() {
                    return Err(BudgetError::RunMismatch);
                }
                if restart.generation() <= suspended.generation() {
                    return Err(BudgetError::StaleGeneration);
                }
                self.state = BudgetState::Running(restart);
                Ok(restart)
            }
            BudgetState::Idle => Err(BudgetError::NoActiveRun),
            _ => Err(BudgetError::NotSuspended),
        }
    }

    pub fn finish(&mut self, run_id: u16, generation: u16) -> Result<(), BudgetError> {
        match self.state {
            BudgetState::Running(run) => {
                if run.run_id() != run_id {
                    return Err(BudgetError::RunMismatch);
                }
                if run.generation() != generation {
                    return Err(BudgetError::GenerationMismatch);
                }
                self.state = BudgetState::Idle;
                Ok(())
            }
            BudgetState::Idle => Err(BudgetError::NoActiveRun),
            _ => Err(BudgetError::NotRunning),
        }
    }
}

impl Default for BudgetController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{BudgetController, BudgetError, BudgetState};
    use crate::choreography::protocol::{BudgetExpired, BudgetRestart, BudgetRun};

    #[test]
    fn budget_controller_admits_only_current_run_expiry() {
        let mut controller = BudgetController::new();
        let run = BudgetRun::new(7, 3, 1000, 123_456);

        assert_eq!(
            controller.admit_expiry(BudgetExpired::new(7, 3)),
            Err(BudgetError::NoActiveRun)
        );
        assert_eq!(controller.start(run), Ok(run));
        assert_eq!(
            controller.admit_expiry(BudgetExpired::new(8, 3)),
            Err(BudgetError::RunMismatch)
        );
        assert_eq!(
            controller.admit_expiry(BudgetExpired::new(7, 4)),
            Err(BudgetError::GenerationMismatch)
        );

        let expired = BudgetExpired::new(7, 3);
        assert_eq!(controller.admit_expiry(expired), Ok(expired));
        assert_eq!(controller.state(), BudgetState::Expired(expired));
    }

    #[test]
    fn budget_controller_requires_expiry_before_suspend_and_suspend_before_restart() {
        let mut controller = BudgetController::new();
        let run = BudgetRun::new(7, 3, 1000, 123_456);
        assert_eq!(controller.start(run), Ok(run));
        assert_eq!(
            controller.suspend_after_expiry(),
            Err(BudgetError::NotExpired)
        );

        let expired = BudgetExpired::new(7, 3);
        assert_eq!(controller.admit_expiry(expired), Ok(expired));
        assert_eq!(controller.suspend_after_expiry(), Ok(expired));

        assert_eq!(
            controller.restart_after_suspend(BudgetRestart::new(7, 3, 1000, 124_000)),
            Err(BudgetError::StaleGeneration)
        );
        let restart = BudgetRestart::new(7, 4, 500, 124_000);
        assert_eq!(controller.restart_after_suspend(restart), Ok(restart));
        assert_eq!(controller.state(), BudgetState::Running(restart));
    }

    #[test]
    fn budget_controller_rejects_zero_fuel_and_finishes_current_run() {
        let mut controller = BudgetController::new();
        assert_eq!(
            controller.start(BudgetRun::new(1, 1, 0, 10)),
            Err(BudgetError::ZeroFuel)
        );

        let run = BudgetRun::new(1, 1, 1, 10);
        assert_eq!(controller.start(run), Ok(run));
        assert_eq!(
            controller.finish(1, 2),
            Err(BudgetError::GenerationMismatch)
        );
        assert_eq!(controller.finish(1, 1), Ok(()));
        assert_eq!(controller.state(), BudgetState::Idle);
    }
}
