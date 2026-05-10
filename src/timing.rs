use crate::model::{ResourceUsage, Step};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimingRecord {
    pub id: String,
    pub duration_ms: u64,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub api_calls: Option<u32>,
    pub cost_usd: Option<f64>,
}

pub fn load_timing(path: &Path) -> Option<HashMap<String, TimingRecord>> {
    let content = fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|p| p.trim()).collect();
        if parts.len() < 2 {
            continue;
        }
        let id = parts[0].to_string();
        let duration_ms = parts[1].parse::<u64>().ok()?;
        let tokens_in = parts.get(2).and_then(|v| v.parse::<u64>().ok());
        let tokens_out = parts.get(3).and_then(|v| v.parse::<u64>().ok());
        let api_calls = parts.get(4).and_then(|v| v.parse::<u32>().ok());
        let cost_usd = parts.get(5).and_then(|v| v.parse::<f64>().ok());

        map.insert(
            id.clone(),
            TimingRecord {
                id,
                duration_ms,
                tokens_in,
                tokens_out,
                api_calls,
                cost_usd,
            },
        );
    }

    Some(map)
}

pub fn apply_timing(steps: &mut [Step], records: &HashMap<String, TimingRecord>) {
    for step in steps.iter_mut() {
        if let Some(record) = records.get(&step.id) {
            step.duration = Some(Duration::from_millis(record.duration_ms));
            let mut usage = ResourceUsage::new();
            if let (Some(i), Some(o)) = (record.tokens_in, record.tokens_out) {
                usage = usage.with_tokens(i, o);
            }
            if let Some(calls) = record.api_calls {
                usage = usage.with_api_calls(calls);
            }
            if let Some(cost) = record.cost_usd {
                usage = usage.with_cost(cost);
            }
            step.resources = usage;
        }
        if !step.children.is_empty() {
            apply_timing(&mut step.children, records);
        }
    }
}
