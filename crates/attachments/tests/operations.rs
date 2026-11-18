use lekalo_laravel_attachments::{
    operations::{check_join, parse_input, Finding, Mode, Recipe, TransactionMode},
    Evidence,
};
const INPUT: &str = include_str!("data/operations.json");
fn super_evidence() -> Evidence {
    serde_json::from_str(include_str!("data/operations-evidence.json")).unwrap()
}
fn evidence() -> Evidence {
    super_evidence()
}
fn finding_codes(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect()
}

#[test]
fn the_fixture_input_parses_into_the_closed_shape() {
    let bytes = INPUT.as_bytes();
    let input = parse_input(bytes).expect("the committed template input parses");
    assert_eq!(input.project_id, "planner");
    assert_eq!(input.namespace_prefix, "Lekalo\\Generated\\Operations");
    assert_eq!(input.operations.len(), 2);
    assert_eq!(input.operations[0].id, "planner.count_focused");
    assert_eq!(input.operations[1].id, "planner.focus_task");
    assert_eq!(input.operations[0].mode, Mode::Managed);
    assert_eq!(input.operations[1].transaction, TransactionMode::Required);
    match input.operations[1].recipe.as_ref().expect("managed recipe") {
        Recipe::SingleEntityUpdate {
            entity, key, kept, ..
        } => {
            assert_eq!(entity.as_str(), "planner.task");
            assert_eq!(key.as_str(), "task_id");
            assert_eq!(kept.as_slice(), ["title"].map(str::to_owned));
        }
        other => panic!("unexpected recipe: {other:?}"),
    }
}

#[test]
fn the_coherent_fixture_joins_clean() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let findings = check_join(&input, &evidence);
    assert!(
        findings.is_empty(),
        "the coherent fixture join must be clean: {findings:?}"
    );
    assert_eq!(evidence.digests["ir"], input.ir_digest);
}

#[test]
fn a_digest_divergence_is_a_binding_finding() {
    let evidence = evidence();
    let mut input = parse_input(INPUT.as_bytes()).unwrap();
    input.ir_digest =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111".to_owned();
    let codes = finding_codes(&check_join(&input, &evidence));
    assert!(
        codes.contains(&"operations.ir-digest".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn a_wrong_reference_kind_is_a_finding() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let mut tampered = input.clone();
    tampered.operations[0].kind = lekalo_laravel_attachments::operations::OperationKind::Command;
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.kind-mismatch".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn undeclared_or_wrong_errors_refuse_against_the_registry_binding() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let mut tampered = input.clone();
    tampered.operations[1].errors = vec!["planner.task_not_found".to_owned()];
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.registry-binding".to_owned()),
        "{codes:?}"
    );
    // The embedded registry carries no binding for an unknown operation.
    tampered.operations[0].id = "planner.unbound_query".to_owned();
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.operation-unresolved".to_owned())
            && codes.contains(&"operations.registry-binding".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn a_query_never_carries_a_write_recipe() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let mut tampered = input.clone();
    tampered.operations[0].recipe = Some(Recipe::SingleEntityUpdate {
        entity: tampered.operations[0].id.clone(),
        key: "task_id".to_owned(),
        assignments: Vec::new(),
        kept: Vec::new(),
        preconditions: Vec::new(),
        missing_error: "planner.store_unavailable".to_owned(),
        emissions: Vec::new(),
    });
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.query-write".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn a_recipe_operand_type_mismatch_is_a_finding() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    // The emit payload operand `{fromEntity: state}` carries the enum,
    // while the event field needs the uuid - an exact type mismatch.
    let mut tampered = input.clone();
    if let Some(Recipe::SingleEntityUpdate { emissions, .. }) =
        tampered.operations[1].recipe.as_mut()
    {
        emissions[0].1.insert(
            "task_id".to_owned(),
            lekalo_laravel_attachments::operations::Operand::FromEntity("state".to_owned()),
        );
    }
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.type-mismatch".to_owned()),
        "{codes:?}"
    );
    // A precondition failure error outside the declared binding set is
    // a registry-binding finding.
    let mut tampered = input.clone();
    if let Some(Recipe::SingleEntityUpdate { preconditions, .. }) =
        tampered.operations[1].recipe.as_mut()
    {
        preconditions[0].error = "planner.unknown_error".to_owned();
    }
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.registry-binding".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn a_policy_binding_must_apply_to_the_operation() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let mut tampered = input.clone();
    tampered.operations[0].policy = Some("planner.deny_bulk_focus".to_owned());
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.policy-unresolved".to_owned()),
        "{codes:?}"
    );
}

#[test]
fn missing_staged_evidence_refuses_before_any_join() {
    let evidence = evidence();
    let mut evidence = evidence;
    evidence.digests.remove("ir");
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let codes = finding_codes(&check_join(&input, &evidence));
    assert_eq!(codes, vec!["operations.evidence-unavailable".to_owned()]);
}

#[test]
fn unchecked_modes_require_the_declared_entrypoint() {
    let evidence = evidence();
    let input = parse_input(INPUT.as_bytes()).unwrap();
    let mut tampered = input.clone();
    tampered.operations[1].mode = Mode::Checked;
    let codes = finding_codes(&check_join(&tampered, &evidence));
    assert!(
        codes.contains(&"operations.entry-required".to_owned()),
        "{codes:?}"
    );
    assert!(!Mode::Checked.emits(), "checked never emits");
    assert!(!Mode::Custom.emits(), "custom never emits");
    assert!(Mode::ScaffoldOnce.emits());
}
