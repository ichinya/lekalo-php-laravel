use lekalo_laravel_attachments::{operations, routes, Evidence};
use serde_json::{json, Value};
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut body = Vec::new();
    io::stdin()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut body)?;
    if body.len() > 8 * 1024 * 1024 {
        return Err("request limit".into());
    }
    let mut q: Value = serde_json::from_slice(&body)?;
    if q["protocol"] != "lekalo/runtime/v1" {
        return Err("protocol".into());
    }
    let result = match q["operation"].as_str() {
        Some("describe") => {
            json!({"id":"lekalo-laravel-attachments","version":env!("CARGO_PKG_VERSION"),"capabilities":[],"rules_api":[],"operations":["attachments.check"]})
        }
        Some("attachments.check") => {
            let input = &mut q["input"];
            if input["schema"] != "lekalo/attachment-check/v1" {
                return Err("input version".into());
            }
            let evidence: Evidence = serde_json::from_value(input["evidence"].take())?;
            let bytes = input["document"]
                .as_str()
                .ok_or("missing document")?
                .as_bytes();
            let findings = match input["family"].as_str() {
                Some("operations") => json!(operations::parse_input(bytes)
                    .map_or_else(|e| e, |i| operations::check_join(&i, &evidence))),
                Some("routes") => json!(routes::parse_input(bytes)
                    .map_or_else(|e| e, |i| routes::check_join(&i, &evidence))),
                _ => return Err("unsupported family".into()),
            };
            json!({"schema":"lekalo/attachment-findings/v1","findings":findings})
        }
        _ => return Err("operation".into()),
    };
    serde_json::to_writer(
        io::stdout().lock(),
        &json!({"protocol":q["protocol"],"request_id":q["request_id"],"result":result}),
    )?;
    Ok(())
}
