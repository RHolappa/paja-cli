// SPDX-License-Identifier: Apache-2.0
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Component, Path},
};

pub const MAX_FILE_BYTES: u64 = 1_048_576;
pub const AGENT_EXAMPLE: &str = include_str!("../examples/agent.json");
pub const SUITE_EXAMPLE: &str = include_str!("../examples/cases.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub schema_version: String,
    pub name: String,
    pub goal: String,
    pub instructions: String,
    pub runtime: Runtime,
    pub tools: Tools,
    pub tests: Tests,
    pub improvement: Improvement,
    pub limits: Limits,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub adapter: String,
    pub model: String,
    pub strategy: Strategy,
    pub request_purchase: bool,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    CatalogOrder,
    Cheapest,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub allowed: Vec<String>,
    pub mode: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tests {
    pub suite: String,
    pub repetitions: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Improvement {
    pub max_attempts: u32,
    pub allow_test_changes: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_trials: u32,
    pub max_tool_calls: u32,
    pub run_timeout_ms: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub schema_version: String,
    pub suite_id: String,
    pub version: u32,
    pub mode: String,
    pub fixtures: Fixtures,
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fixtures {
    pub catalog: Vec<Item>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub price_cents: u32,
    pub currency: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub input: Input,
    pub assertions: Vec<Assertion>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub count: u32,
    pub budget_cents: u32,
    pub purchase: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Assertion {
    SelectedItemsFromFixture,
    SelectedItemCount { equals: u32 },
    DistinctItemCount { equals: u32 },
    FixturePriceTotalAtMost { amount_cents: u32 },
    OutputTotalMatchesFixture,
    ToolCalled { tool: String },
    NoDisallowedToolRequests,
    PurchaseNotPerformed,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub schema_version: String,
    pub agent: Agent,
    pub default_suite: Suite,
}

fn text_ok(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max
}
pub fn relative_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_) | Component::CurDir))
    {
        return Err(Error::invalid("File paths must stay inside the project"));
    }
    Ok(())
}
pub fn read_json<T: serde::de::DeserializeOwned>(root: &Path, name: &str) -> Result<T> {
    relative_path(name)?;
    let root = root.canonicalize()?;
    let path = root.join(name).canonicalize()?;
    if !path.starts_with(root) {
        return Err(Error::invalid("File resolves outside the project"));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(Error::invalid("Input exceeds the 1 MiB limit"));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
pub fn digest<T: Serialize>(value: &T) -> Result<String> {
    let bytes = Sha256::digest(serde_json::to_vec(value)?);
    let mut hex = String::with_capacity(64);
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        hex.push(char::from(DIGITS[usize::from(byte >> 4)]));
        hex.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    Ok(hex)
}
impl Agent {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != "paja.agent.v1" {
            return Err(Error::invalid("Unsupported agent schema"));
        }
        if !text_ok(&self.name, 100)
            || !text_ok(&self.goal, 4096)
            || !text_ok(&self.instructions, 16384)
        {
            return Err(Error::invalid(
                "Name, goal, and instructions must be nonempty and bounded",
            ));
        }
        if self.runtime.adapter != "fixture"
            || self.runtime.model != "deterministic-control"
            || self.tools.mode != "fixture"
        {
            return Err(Error::new(
                "unsupported_capability",
                "Only the fixture/deterministic-control adapter is implemented; no live model fallback",
                2,
            ));
        }
        let tools: BTreeSet<_> = self.tools.allowed.iter().map(String::as_str).collect();
        if tools.len() != self.tools.allowed.len()
            || tools
                .iter()
                .any(|t| !matches!(*t, "catalog.read" | "calculate"))
        {
            return Err(Error::invalid(
                "Fixture tools may only include catalog.read and calculate, without duplicates",
            ));
        }
        relative_path(&self.tests.suite)?;
        if !(1..=100).contains(&self.tests.repetitions)
            || self.improvement.max_attempts != 1
            || self.improvement.allow_test_changes
        {
            return Err(Error::invalid(
                "Use 1–100 repetitions, exactly one improvement attempt, and allow_test_changes=false",
            ));
        }
        if !(1..=512).contains(&self.limits.max_trials)
            || self.limits.max_tool_calls > 4096
            || !(1..=10_000).contains(&self.limits.run_timeout_ms)
        {
            return Err(Error::invalid(
                "Limits: 1–512 trials, 0–4096 tool calls, 1–10000 ms",
            ));
        }
        Ok(())
    }
}
impl Input {
    pub fn validate(&self) -> Result<()> {
        if !(1..=20).contains(&self.count) || self.budget_cents > 100_000_000 {
            return Err(Error::invalid(
                "Fixture input needs count 1–20 and budget_cents <= 100000000",
            ));
        }
        Ok(())
    }
    pub fn parse(text: &str) -> Result<Self> {
        let input: Self = if text.trim_start().starts_with('{') {
            serde_json::from_str(text)?
        } else {
            match text
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase()
                .as_str()
            {
                "choose two lunches for a total of eight euros"
                | "choose two lunches for a total of 8 euros" => Self {
                    count: 2,
                    budget_cents: 800,
                    purchase: false,
                },
                "choose a lunch and buy it for me" => Self {
                    count: 1,
                    budget_cents: 800,
                    purchase: true,
                },
                _ => {
                    return Err(Error::invalid(
                        "The control fixture accepts the two documented demo phrases or JSON with count, budget_cents, and purchase. It does not understand arbitrary language.",
                    ));
                }
            }
        };
        input.validate()?;
        Ok(input)
    }
}
impl Suite {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != "paja.tests.v1"
            || self.mode != "fixture"
            || self.version == 0
            || !text_ok(&self.suite_id, 100)
        {
            return Err(Error::invalid(
                "Unsupported suite schema, mode, ID, or version",
            ));
        }
        if self.cases.is_empty()
            || self.cases.len() > 64
            || self.fixtures.catalog.is_empty()
            || self.fixtures.catalog.len() > 128
        {
            return Err(Error::invalid(
                "A suite needs 1–64 cases and 1–128 catalog items",
            ));
        }
        let mut ids = BTreeSet::new();
        for item in &self.fixtures.catalog {
            if !text_ok(&item.id, 100)
                || !ids.insert(&item.id)
                || item.currency != "EUR"
                || item.price_cents > 1_000_000
            {
                return Err(Error::invalid(
                    "Catalog items need unique IDs, EUR currency, and bounded integer cents",
                ));
            }
        }
        ids.clear();
        for case in &self.cases {
            case.input.validate()?;
            if !text_ok(&case.id, 100)
                || !ids.insert(&case.id)
                || case.assertions.is_empty()
                || case.assertions.len() > 32
            {
                return Err(Error::invalid("Cases need unique IDs and 1–32 assertions"));
            }
            for assertion in &case.assertions {
                match assertion {
                    Assertion::ToolCalled { tool }
                        if !matches!(tool.as_str(), "catalog.read" | "calculate") =>
                    {
                        return Err(Error::invalid("Unknown assertion tool"));
                    }
                    Assertion::SelectedItemCount { equals }
                    | Assertion::DistinctItemCount { equals }
                        if *equals > 20 =>
                    {
                        return Err(Error::invalid("Assertion count exceeds fixture limit"));
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }
}
