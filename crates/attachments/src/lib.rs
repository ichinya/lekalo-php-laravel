//! Framework-owned semantic validation over the canonical data supplied by Lekalo.
pub mod operations;
pub mod routes;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Evidence {
    pub project: String,
    pub ir: CompiledProject,
    pub digests: BTreeMap<String, String>,
    pub registry: Option<BTreeMap<String, Binding>>,
    pub transport: Option<Transport>,
    pub transport_errors: Vec<String>,
}
#[derive(Deserialize)]
pub struct Binding {
    pub kind: String,
    pub errors: Vec<String>,
}
#[derive(Deserialize)]
pub struct Transport {
    #[serde(rename = "projectId")]
    pub project: String,
    #[serde(rename = "irRef")]
    pub ir: IrRef,
    #[serde(rename = "securitySchemes", default)]
    pub schemes: Vec<Scheme>,
    pub endpoints: Vec<Endpoint>,
}
#[derive(Deserialize)]
pub struct IrRef {
    pub digest: String,
}
#[derive(Deserialize)]
pub struct Scheme {
    pub id: String,
}
#[derive(Deserialize)]
pub struct Endpoint {
    pub endpoint: String,
}
#[derive(Deserialize)]
pub struct CompiledProject {
    definitions: Vec<Definition>,
}
#[derive(Deserialize)]
struct Field {
    name: String,
    r#type: TypeRef,
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TypeRef {
    Ref(String),
    List(Box<TypeRef>),
    Optional(Box<TypeRef>),
}
#[derive(Deserialize)]
struct EnumValue {
    value: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Definition {
    Command(Command),
    Query(Common),
    Enum(Enum),
    Policy(Policy),
    Entity(Entity),
    Effect(Effect),
    Event(Event),
    Endpoint(EndpointDef),
    Scalar(Common),
    ValueObject(Common),
    Relation(Common),
    Scenario(Common),
    TargetBinding(Common),
}
#[derive(Deserialize)]
struct Common {
    id: String,
}
#[derive(Deserialize)]
struct Command {
    id: String,
    #[serde(default)]
    input: Vec<Field>,
    #[serde(default)]
    effects: Vec<String>,
}
#[derive(Deserialize)]
struct Enum {
    id: String,
    values: Vec<EnumValue>,
}
#[derive(Deserialize)]
struct Policy {
    id: String,
    #[serde(default)]
    applies_to: Vec<String>,
}
#[derive(Deserialize)]
struct Entity {
    id: String,
    fields: Vec<Field>,
    identity: Vec<String>,
}
#[derive(Deserialize)]
struct Effect {
    id: String,
    #[serde(default)]
    emits: Vec<String>,
}
#[derive(Deserialize)]
struct Event {
    id: String,
    #[serde(default)]
    payload: Vec<Field>,
}
#[derive(Deserialize)]
struct EndpointDef {
    id: String,
    invokes: String,
}
impl Definition {
    fn identity(&self) -> (&str, &str) {
        match self {
            Self::Command(x) => (&x.id, "command"),
            Self::Query(x) => (&x.id, "query"),
            Self::Enum(x) => (&x.id, "enum"),
            Self::Policy(x) => (&x.id, "policy"),
            Self::Entity(x) => (&x.id, "entity"),
            Self::Effect(x) => (&x.id, "effect"),
            Self::Event(x) => (&x.id, "event"),
            Self::Endpoint(x) => (&x.id, "endpoint"),
            Self::Scalar(x) => (&x.id, "scalar"),
            Self::ValueObject(x) => (&x.id, "value-object"),
            Self::Relation(x) => (&x.id, "relation"),
            Self::Scenario(x) => (&x.id, "scenario"),
            Self::TargetBinding(x) => (&x.id, "target-binding"),
        }
    }
}
