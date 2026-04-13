use crate::projects::documents::{apply_merge_action, merge_action_target, ordered_merge_actions};
use ralph_core::prd::{Prd, UserStory};
use ralph_core::WorktreeCompletion;

#[test]
fn parallel_completions_produce_one_ordered_write_sequence() {
    let root = std::env::temp_dir().join(format!("loopforge-merge-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    seed_prd(&root);

    let actions = ordered_merge_actions(vec![
        completion(
            "S-002",
            "wt-b",
            false,
            true,
            1,
            Some("guardrail from wt-b"),
            Some("bbb222"),
        ),
        completion("S-001", "wt-a", true, false, 0, None, Some("aaa111")),
    ]);

    let mut targets = Vec::new();
    for action in &actions {
        targets.push(merge_action_target(&root, action));
        apply_merge_action(&root, action).unwrap();
    }

    assert_eq!(
        targets,
        vec![
            root.join("prd.json").display().to_string(),
            "session:wt-a".to_string(),
            root.join("prd.json").display().to_string(),
            root.join("guardrails.md").display().to_string(),
            "session:wt-b".to_string(),
        ]
    );

    let prd = Prd::load(&root.join("prd.json")).unwrap();
    assert!(
        prd.stories
            .iter()
            .find(|story| story.id == "S-001")
            .unwrap()
            .passes
    );
    let blocked = prd
        .stories
        .iter()
        .find(|story| story.id == "S-002")
        .unwrap();
    assert!(!blocked.passes);
    assert!(blocked.blocked);
    assert_eq!(
        std::fs::read_to_string(root.join("guardrails.md")).unwrap(),
        "guardrail from wt-b\n"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn merge_targets_keep_contract_artifact_filenames() {
    let root = std::env::temp_dir().join(format!("loopforge-targets-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();

    let actions = ordered_merge_actions(vec![completion(
        "S-003",
        "wt-c",
        false,
        true,
        0,
        Some("guardrail from wt-c"),
        Some("ccc333"),
    )]);

    let targets: Vec<String> = actions
        .iter()
        .map(|action| merge_action_target(&root, action))
        .collect();

    assert!(targets.iter().any(|target| target.ends_with("prd.json")));
    assert!(targets
        .iter()
        .any(|target| target.ends_with("guardrails.md")));
    assert!(targets.iter().any(|target| target == "session:wt-c"));

    let _ = std::fs::remove_dir_all(root);
}

fn seed_prd(root: &std::path::Path) {
    let prd = Prd {
        project_name: "Merge Contract".to_string(),
        feature: String::new(),
        working_directory: String::new(),
        branch_name: None,
        stories: vec![story("S-001"), story("S-002")],
        generated_at: None,
    };
    prd.save(&root.join("prd.json")).unwrap();
    std::fs::write(root.join("guardrails.md"), "").unwrap();
}

fn story(id: &str) -> UserStory {
    UserStory {
        id: id.to_string(),
        title: id.to_string(),
        description: None,
        acceptance_criteria: vec!["contract".to_string()],
        scope: Default::default(),
        verification: Default::default(),
        commit_message: None,
        priority: Default::default(),
        estimated_complexity: Default::default(),
        estimated_minutes: 0,
        depends_on: vec![],
        passes: false,
        blocked: false,
        attempts: 0,
        notes: None,
    }
}

fn completion(
    story_id: &str,
    worktree_id: &str,
    passed: bool,
    blocked: bool,
    sequence: u64,
    guardrail_append: Option<&str>,
    head_commit: Option<&str>,
) -> WorktreeCompletion {
    WorktreeCompletion {
        worktree_id: worktree_id.to_string(),
        story_id: story_id.to_string(),
        passed,
        blocked,
        head_commit: head_commit.map(str::to_string),
        guardrail_append: guardrail_append.map(str::to_string),
        sequence,
    }
}
