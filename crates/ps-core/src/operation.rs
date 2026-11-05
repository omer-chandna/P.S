use crate::error::{PsError, PsResult};
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct ActiveOperation {
    id: Uuid,
    cancel: CancellationToken,
}

/// Ensures only one optimization runs at a time and lets the UI cancel it.
#[derive(Default)]
pub struct OperationGuard {
    inner: Mutex<Option<ActiveOperation>>,
}

pub struct OperationLease {
    id: Uuid,
    cancel: CancellationToken,
    guard: std::sync::Arc<OperationGuard>,
}

impl OperationGuard {
    pub fn try_begin(self: &std::sync::Arc<Self>) -> PsResult<OperationLease> {
        let mut slot = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if slot.is_some() {
            return Err(PsError::AlreadyRunning);
        }
        let id = Uuid::new_v4();
        let cancel = CancellationToken::new();
        *slot = Some(ActiveOperation {
            id,
            cancel: cancel.clone(),
        });
        Ok(OperationLease {
            id,
            cancel,
            guard: std::sync::Arc::clone(self),
        })
    }

    pub fn cancel_current(&self) -> bool {
        let slot = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = slot.as_ref() {
            active.cancel.cancel();
            true
        } else {
            false
        }
    }

    pub fn is_current(&self, id: Uuid) -> bool {
        let slot = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        slot.as_ref().is_some_and(|active| active.id == id)
    }

    fn finish(&self, id: Uuid) {
        let mut slot = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if slot.as_ref().is_some_and(|active| active.id == id) {
            *slot = None;
        }
    }
}

impl OperationLease {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

impl Drop for OperationLease {
    fn drop(&mut self) {
        self.guard.finish(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_overlapping_operations_until_the_first_finishes() {
        let guard = std::sync::Arc::new(OperationGuard::default());
        let first = guard.try_begin().unwrap();
        assert!(matches!(guard.try_begin(), Err(PsError::AlreadyRunning)));
        let id = first.id();
        drop(first);
        let second = guard.try_begin().unwrap();
        assert_ne!(second.id(), id);
    }

    #[test]
    fn cancel_marks_the_active_lease() {
        let guard = std::sync::Arc::new(OperationGuard::default());
        let lease = guard.try_begin().unwrap();
        assert!(guard.cancel_current());
        assert!(lease.is_cancelled());
        assert!(!guard.cancel_current() || lease.is_cancelled());
    }
}
