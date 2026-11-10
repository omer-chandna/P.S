use crate::optimizer::OptimizationStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What the desktop shell should do after a validated optimization.
/// Unchanged results never paste. Suspicious rewrites preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryPlan {
    Replace,
    LeaveUnchanged,
    Preview,
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryContext {
    pub manual: bool,
    pub auto_replace: bool,
    pub preview_before_replace: bool,
    pub target_still_valid: Option<bool>,
    pub selection_still_matches: Option<bool>,
}

pub fn plan_delivery(
    status: OptimizationStatus,
    suspicious: bool,
    context: &DeliveryContext,
) -> DeliveryPlan {
    match status {
        OptimizationStatus::Unchanged => DeliveryPlan::LeaveUnchanged,
        OptimizationStatus::Rewritten if context.manual => DeliveryPlan::Preview,
        OptimizationStatus::Rewritten => {
            let safe_to_paste = !suspicious
                && context.auto_replace
                && !context.preview_before_replace
                && context.target_still_valid == Some(true)
                && context.selection_still_matches == Some(true);
            if safe_to_paste {
                DeliveryPlan::Replace
            } else {
                DeliveryPlan::Preview
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowTarget {
    pub app_name: String,
    pub window_title: String,
    pub pid: u32,
    pub native_id: i64,
}

impl WindowTarget {
    pub fn same_target(&self, other: &Self) -> bool {
        self.pid == other.pid && self.app_name == other.app_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingReplacement {
    pub id: Uuid,
    pub original_text: String,
    pub rewritten_prompt: String,
    pub target: WindowTarget,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(auto_replace: bool, preview: bool) -> DeliveryContext {
        DeliveryContext {
            manual: false,
            auto_replace,
            preview_before_replace: preview,
            target_still_valid: Some(true),
            selection_still_matches: Some(true),
        }
    }

    #[test]
    fn unchanged_results_never_replace() {
        let context = ready(true, false);
        assert_eq!(
            plan_delivery(OptimizationStatus::Unchanged, false, &context),
            DeliveryPlan::LeaveUnchanged
        );
    }

    #[test]
    fn safe_rewrite_replaces_only_when_the_target_is_intact() {
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &ready(true, false)),
            DeliveryPlan::Replace
        );
        let mut moved = ready(true, false);
        moved.target_still_valid = Some(false);
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &moved),
            DeliveryPlan::Preview
        );
        let mut changed = ready(true, false);
        changed.selection_still_matches = Some(false);
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &changed),
            DeliveryPlan::Preview
        );
        let mut unknown = ready(true, false);
        unknown.target_still_valid = None;
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &unknown),
            DeliveryPlan::Preview
        );
    }

    #[test]
    fn suspicious_preview_and_manual_runs_do_not_auto_replace() {
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, true, &ready(true, false)),
            DeliveryPlan::Preview
        );
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &ready(true, true)),
            DeliveryPlan::Preview
        );
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &ready(false, false)),
            DeliveryPlan::Preview
        );
        let mut manual = ready(true, false);
        manual.manual = true;
        assert_eq!(
            plan_delivery(OptimizationStatus::Rewritten, false, &manual),
            DeliveryPlan::Preview
        );
        assert_eq!(
            plan_delivery(OptimizationStatus::Unchanged, false, &manual),
            DeliveryPlan::LeaveUnchanged
        );
    }
}
