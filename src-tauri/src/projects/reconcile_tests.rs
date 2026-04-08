use super::reconcile::reconcile_prd_state;
use ralph_core::prd::Prd;
use std::collections::HashSet;

fn sample_prd() -> Prd {
    serde_json::from_str(
        r#"{
            "projectName":"LoopForge",
            "stories":[
                {"id":"S-001","title":"One","acceptanceCriteria":["a"],"passes":false,"blocked":false},
                {"id":"S-002","title":"Two","acceptanceCriteria":["b"],"passes":false,"blocked":false},
                {"id":"S-003","title":"Three","acceptanceCriteria":["c"],"passes":false,"blocked":true}
            ]
        }"#,
    )
    .unwrap()
}

#[test]
fn reconcile_marks_successful_stories_as_passed() {
    let mut prd = sample_prd();
    let successful = HashSet::from(["S-001".to_string(), "S-002".to_string()]);
    let blocked = HashSet::new();

    let changed = reconcile_prd_state(&mut prd, &successful, &blocked);

    assert!(changed);
    assert!(prd.stories[0].passes);
    assert!(prd.stories[1].passes);
    assert!(!prd.stories[0].blocked);
    assert!(!prd.stories[1].blocked);
}

#[test]
fn reconcile_rebuilds_blocked_state_from_current_evidence() {
    let mut prd = sample_prd();
    let successful = HashSet::from(["S-001".to_string()]);
    let blocked = HashSet::from(["S-002".to_string()]);

    reconcile_prd_state(&mut prd, &successful, &blocked);

    assert!(prd.stories[0].passes);
    assert!(!prd.stories[0].blocked);
    assert!(!prd.stories[1].passes);
    assert!(prd.stories[1].blocked);
    assert!(!prd.stories[2].blocked);
}
