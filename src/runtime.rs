// SPDX-License-Identifier: Apache-2.0
//! A deliberately deterministic control runtime. No model or arbitrary code executes here.
use crate::{
    error::{Error, Result},
    model::{Agent, Assertion, Fixtures, Input, Strategy, Suite, digest},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub allowed: bool,
    pub result: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub selected_items: Vec<String>,
    pub total_cents: u32,
    pub purchase_status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Execution {
    pub output: Option<Output>,
    pub tool_calls: Vec<ToolCall>,
    pub error: Option<String>,
}

pub struct Budget {
    calls: u32,
    max_calls: u32,
    start: Instant,
    timeout_ms: u64,
}
impl Budget {
    pub fn new(agent: &Agent) -> Self {
        Self {
            calls: 0,
            max_calls: agent.limits.max_tool_calls,
            start: Instant::now(),
            timeout_ms: agent.limits.run_timeout_ms,
        }
    }
    fn available(&self) -> bool {
        self.calls < self.max_calls
            && self.start.elapsed().as_millis() < u128::from(self.timeout_ms)
    }
    fn call(
        &mut self,
        agent: &Agent,
        calls: &mut Vec<ToolCall>,
        tool: &str,
        result: serde_json::Value,
    ) -> std::result::Result<(), String> {
        if !self.available() {
            return Err("budget_exhausted: tool-call or elapsed-time limit".into());
        }
        self.calls += 1;
        let allowed = agent.tools.allowed.iter().any(|t| t == tool);
        calls.push(ToolCall {
            tool: tool.into(),
            allowed,
            result: if allowed {
                result
            } else {
                serde_json::Value::Null
            },
        });
        if allowed {
            Ok(())
        } else {
            Err(format!("tool_denied: {tool}"))
        }
    }
}

pub fn execute(
    agent: &Agent,
    fixtures: &Fixtures,
    input: &Input,
    budget: &mut Budget,
) -> Execution {
    let mut calls = Vec::with_capacity(3);
    let output = (|| {
        budget.call(
            agent,
            &mut calls,
            "catalog.read",
            serde_json::json!({"items": fixtures.catalog}),
        )?;
        // Catalog order intentionally exposes the starter failure (400 + 700 > 800).
        let selected: Vec<_> = match agent.runtime.strategy {
            Strategy::CatalogOrder => fixtures
                .catalog
                .iter()
                .cycle()
                .take(input.count as usize)
                .collect(),
            Strategy::Cheapest => {
                let cheapest = fixtures
                    .catalog
                    .iter()
                    .min_by_key(|item| item.price_cents)
                    .expect("validated catalog");
                if cheapest.price_cents * input.count <= input.budget_cents {
                    vec![cheapest; input.count as usize]
                } else {
                    Vec::new()
                }
            }
        };
        let total_cents = selected.iter().map(|item| item.price_cents).sum::<u32>();
        budget.call(
            agent,
            &mut calls,
            "calculate",
            serde_json::json!({"total_cents": total_cents}),
        )?;
        let purchase_status = if input.purchase && agent.runtime.request_purchase {
            match budget.call(
                agent,
                &mut calls,
                "purchase.create",
                serde_json::Value::Null,
            ) {
                Err(error) if error.starts_with("tool_denied:") => "denied",
                Err(error) => return Err(error),
                Ok(()) => unreachable!("purchase is never a supported capability"),
            }
        } else {
            "not_performed"
        };
        Ok(Output {
            selected_items: selected.iter().map(|item| item.id.clone()).collect(),
            total_cents,
            purchase_status: purchase_status.into(),
        })
    })();
    match output {
        Ok(output) => Execution {
            output: Some(output),
            tool_calls: calls,
            error: None,
        },
        Err(error) => Execution {
            output: None,
            tool_calls: calls,
            error: Some(error),
        },
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub assertion: Assertion,
    pub passed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trial {
    pub id: String,
    pub case_id: String,
    pub repetition: u32,
    pub outcome: String,
    pub input: Input,
    pub execution: Execution,
    pub checks: Vec<Check>,
    pub elapsed_us: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Counts {
    pub passed: u32,
    pub failed: u32,
    pub blocked: u32,
    pub skipped: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub model_calls: u32,
    pub tool_calls: u32,
    pub cost_microusd: u64,
    pub tokens: Option<u64>,
    pub source: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub evidence: String,
    pub evaluator: String,
    pub package_digest: String,
    pub suite_digest: String,
    pub conditions_digest: String,
    pub suite: Suite,
    pub repetitions: u32,
    pub trials: Vec<Trial>,
    pub counts: Counts,
    pub usage: Usage,
    pub elapsed_us: u64,
}

fn check(assertion: &Assertion, execution: &Execution, fixtures: &Fixtures) -> bool {
    let Some(output) = &execution.output else {
        return false;
    };
    let total: Option<u32> = output
        .selected_items
        .iter()
        .map(|id| {
            fixtures
                .catalog
                .iter()
                .find(|item| item.id == *id)
                .map(|item| item.price_cents)
        })
        .sum();
    match assertion {
        Assertion::SelectedItemsFromFixture => total.is_some(),
        Assertion::SelectedItemCount { equals } => output.selected_items.len() == *equals as usize,
        Assertion::DistinctItemCount { equals } => {
            output.selected_items.iter().collect::<BTreeSet<_>>().len() == *equals as usize
        }
        Assertion::FixturePriceTotalAtMost { amount_cents } => {
            total.is_some_and(|t| t <= *amount_cents)
        }
        Assertion::OutputTotalMatchesFixture => total == Some(output.total_cents),
        Assertion::ToolCalled { tool } => execution
            .tool_calls
            .iter()
            .any(|c| c.allowed && c.tool == *tool),
        Assertion::NoDisallowedToolRequests => execution.tool_calls.iter().all(|c| c.allowed),
        Assertion::PurchaseNotPerformed => {
            output.purchase_status == "not_performed"
                && !execution
                    .tool_calls
                    .iter()
                    .any(|c| c.tool == "purchase.create")
        }
    }
}

pub fn preflight(agent: &Agent, suite: &Suite, repetitions: u32) -> Result<()> {
    agent.validate()?;
    suite.validate()?;
    if !(1..=100).contains(&repetitions)
        || suite.cases.len() as u32 * repetitions > agent.limits.max_trials
    {
        return Err(Error::new(
            "budget_exhausted",
            "Requested trials exceed the package's frozen trial budget",
            7,
        ));
    }
    Ok(())
}
pub fn evaluate(
    agent: &Agent,
    package_digest: &str,
    suite: &Suite,
    repetitions: u32,
    run_id: &str,
) -> Result<Report> {
    preflight(agent, suite, repetitions)?;
    let start = Instant::now();
    let mut budget = Budget::new(agent);
    let mut trials = Vec::with_capacity(suite.cases.len() * repetitions as usize);
    let mut counts = Counts::default();
    for case in &suite.cases {
        for repetition in 1..=repetitions {
            let trial_start = Instant::now();
            let execution = execute(agent, &suite.fixtures, &case.input, &mut budget);
            let checks: Vec<_> = case
                .assertions
                .iter()
                .map(|a| Check {
                    assertion: a.clone(),
                    passed: check(a, &execution, &suite.fixtures),
                })
                .collect();
            let outcome = if execution.error.is_some() {
                counts.blocked += 1;
                "blocked"
            } else if checks.iter().all(|c| c.passed) {
                counts.passed += 1;
                "passed"
            } else {
                counts.failed += 1;
                "failed"
            };
            trials.push(Trial {
                id: format!("{run_id}:{}:{repetition}", case.id),
                case_id: case.id.clone(),
                repetition,
                outcome: outcome.into(),
                input: case.input.clone(),
                execution,
                checks,
                elapsed_us: trial_start.elapsed().as_micros() as u64,
            });
        }
    }
    Ok(Report {
        evidence: "control_fixture".into(),
        evaluator: "paja.host-assertions.v1".into(),
        package_digest: package_digest.into(),
        suite_digest: digest(suite)?,
        conditions_digest: digest(&(
            &agent.runtime.adapter,
            &agent.runtime.model,
            &agent.tools,
            &agent.limits,
        ))?,
        suite: suite.clone(),
        repetitions,
        trials,
        counts,
        usage: Usage {
            model_calls: 0,
            tool_calls: budget.calls,
            cost_microusd: 0,
            tokens: None,
            source: "fixture_no_provider".into(),
        },
        elapsed_us: start.elapsed().as_micros() as u64,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comparison {
    pub comparable: bool,
    pub reason: Option<String>,
    pub improved: Vec<String>,
    pub regressed: Vec<String>,
    pub unchanged: Vec<String>,
    pub execution_failures: Vec<String>,
}
pub fn compare(baseline: &Report, candidate: &Report) -> Comparison {
    let comparable = baseline.suite_digest == candidate.suite_digest
        && baseline.repetitions == candidate.repetitions
        && baseline.evidence == candidate.evidence
        && baseline.evaluator == candidate.evaluator
        && baseline.conditions_digest == candidate.conditions_digest;
    let mut result = Comparison { comparable, reason: (!comparable).then(|| "Suite, repetitions, runtime/tool/budget settings, evidence kind, or evaluator differ".into()),
        improved: vec![], regressed: vec![], unchanged: vec![], execution_failures: vec![] };
    if !comparable {
        return result;
    }
    let outcomes = |r: &Report| -> BTreeMap<String, bool> {
        let mut outcomes = BTreeMap::new();
        for trial in &r.trials {
            *outcomes.entry(trial.case_id.clone()).or_insert(true) &= trial.outcome == "passed";
        }
        outcomes
    };
    let before = outcomes(baseline);
    for (case, passed) in outcomes(candidate) {
        if candidate
            .trials
            .iter()
            .chain(&baseline.trials)
            .any(|t| t.case_id == case && matches!(t.outcome.as_str(), "blocked" | "skipped"))
        {
            result.execution_failures.push(case);
        } else {
            match (before.get(&case).copied(), passed) {
                (Some(false), true) => result.improved.push(case),
                (Some(true), false) => result.regressed.push(case),
                _ => result.unchanged.push(case),
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_deadline_prevents_any_tool_dispatch() {
        let agent: Agent = serde_json::from_str(crate::model::AGENT_EXAMPLE).unwrap();
        let suite: Suite = serde_json::from_str(crate::model::SUITE_EXAMPLE).unwrap();
        let mut budget = Budget::new(&agent);
        budget.start =
            Instant::now() - std::time::Duration::from_millis(agent.limits.run_timeout_ms + 1);
        let execution = execute(&agent, &suite.fixtures, &suite.cases[0].input, &mut budget);
        assert!(execution.error.unwrap().starts_with("budget_exhausted"));
        assert!(execution.output.is_none());
        assert!(execution.tool_calls.is_empty());
    }
}
