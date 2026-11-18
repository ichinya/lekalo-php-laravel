use lekalo_laravel_attachments::{
    routes::{check_join, parse_input, Finding, Mode},
    Evidence,
};
const INPUT: &str = include_str!("data/routes.json");
fn super_evidence() -> Evidence {
    serde_json::from_str(include_str!("data/routes-evidence.json")).unwrap()
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

/// Parse the (possibly tampered) input bytes into the typed shape.
fn typed(bytes: &str) -> lekalo_laravel_attachments::routes::RoutesInput {
    parse_input(bytes.as_bytes()).expect("the tampered input stays shape-valid")
}

#[test]
fn the_committed_template_parses_into_the_closed_shape() {
    let bytes = INPUT.as_bytes();
    let input = parse_input(bytes).expect("the committed template input parses");
    assert_eq!(input.project_id, "planner");
    assert_eq!(input.namespace_prefix, "Lekalo\\Generated\\Routes");
    assert_eq!(input.middleware.len(), 1);
    assert_eq!(input.middleware[0].scheme, "user_bearer");
    assert_eq!(input.middleware[0].middleware, "fixture.auth");
    assert_eq!(input.routes.len(), 13);
    assert_eq!(input.routes[0].id, "planner.endpoint_backlog");
    assert_eq!(input.routes[0].mode, Mode::Managed);
    assert_eq!(input.routes[9].id, "planner.endpoint_tasks_focus");
    assert_eq!(input.routes[9].mode, Mode::Checked);
    let entry = input.routes[9].entry.as_ref().expect("checked entry");
    assert_eq!(entry.fqn, "App\\Http\\Controllers\\TaskFocusController");
    assert_eq!(entry.method, "focus");
    assert_eq!(input.routes[12].id, "planner.endpoint_unplan");
}

#[test]
fn the_accepted_join_is_empty_over_the_staged_fixture() {
    let evidence = evidence();
    let bytes = INPUT;
    let input = parse_input(bytes.as_bytes()).expect("input parses");
    let findings = check_join(&input, &evidence);
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn a_missing_home_refuses_as_evidence_unavailable() {
    let mut evidence = evidence();
    evidence.digests.clear();
    let bytes = INPUT;
    let input = parse_input(bytes.as_bytes()).expect("input parses");
    let findings = check_join(&input, &evidence);
    assert_eq!(
        finding_codes(&findings),
        vec!["routes.evidence-unavailable"]
    );
}

#[test]
fn diverging_digests_are_typed_findings() {
    let evidence = evidence();
    let bytes = INPUT;
    // Tamper with each pin one at a time.
    for (member, expected) in [
        ("irDigest", "routes.ir-digest"),
        ("transportDigest", "routes.transport-digest"),
        ("typesInputDigest", "routes.types-unbound"),
        ("operationsInputDigest", "routes.operations-unbound"),
    ] {
        let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
        value[member] = serde_json::Value::String(format!("sha256:{}", "f".repeat(64)));
        let tampered = typed(&value.to_string());
        let findings = check_join(&tampered, &evidence);
        assert!(
            finding_codes(&findings).contains(&expected.to_owned()),
            "{member}: {findings:?}"
        );
    }
}

#[test]
fn unresolved_symbols_and_broken_pairings_are_typed_findings() {
    let evidence = evidence();
    let bytes = INPUT;

    // A foreign endpoint id (aaa keeps the canonical array order).
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["routes"][0]["id"] = serde_json::Value::String("planner.endpoint_aaa".to_owned());
    value["routes"][0]["operation"] = serde_json::Value::String("planner.focus_task".to_owned());
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.endpoint-unresolved".to_owned()),
        "a foreign endpoint refuses: {findings:?}"
    );

    // An operation the endpoint never invokes.
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["routes"][0]["operation"] = serde_json::Value::String("planner.list_tasks".to_owned());
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.invokes-mismatch".to_owned()),
        "an invokes mismatch refuses: {findings:?}"
    );

    // A non-command/query operation symbol.
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["routes"][0]["operation"] = serde_json::Value::String("planner.task".to_owned());
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.invokes-mismatch".to_owned()),
        "a foreign operation symbol refuses: {findings:?}"
    );

    // Checked without entry (the checked record rides index 9 of the
    // canonical id order).
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["routes"][9] = serde_json::json!({
        "id": "planner.endpoint_tasks_focus",
        "operation": "planner.focus_task",
        "mode": "checked",
    });
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.entry-required".to_owned()),
        "checked without entry refuses: {findings:?}"
    );

    // Managed with a declared entry.
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["routes"][0]["entry"] = serde_json::json!({
        "fqn": "App\\Http\\Controllers\\TaskFocusController",
        "method": "focus",
    });
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.entry-required".to_owned()),
        "managed with a foreign entry refuses: {findings:?}"
    );
}

#[test]
fn an_unbound_endpoint_and_a_foreign_scheme_are_typed_findings() {
    let evidence = evidence();
    let bytes = INPUT;

    // Drop the today binding from the transport home: the route record
    // no longer joins the wire.
    let mut evidence = evidence;
    evidence
        .transport
        .as_mut()
        .unwrap()
        .endpoints
        .retain(|e| e.endpoint != "planner.endpoint_today");
    let value: serde_json::Value = serde_json::from_str(bytes).unwrap();
    let input = typed(&value.to_string());
    let findings = check_join(&input, &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.transport-unbound".to_owned()),
        "a dropped wire binding refuses: {findings:?}"
    );

    // A middleware scheme the attachment never declares.
    let evidence = super_evidence();
    let mut value: serde_json::Value = serde_json::from_str(bytes).expect("raw input");
    value["policy"]["middleware"] = serde_json::json!([
        {"scheme": "svc_apikey", "middleware": "fixture.auth"},
    ]);
    let findings = check_join(&typed(&value.to_string()), &evidence);
    assert!(
        finding_codes(&findings).contains(&"routes.scheme-unresolved".to_owned()),
        "a foreign scheme mapping refuses: {findings:?}"
    );
}
