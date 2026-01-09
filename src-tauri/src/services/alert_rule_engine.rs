use crate::storage::temporal::TemporalEvent;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use regex::Regex;
use chrono::{TimeZone, Utc, Datelike, Timelike};

pub struct AlertRuleEngine;

impl AlertRuleEngine {
    /// Enhanced rule matching with support for complex conditions
    pub fn rule_matches(
        rule_json: &Value,
        haystack_lower: &str,
        entities_lower: &HashSet<String>,
        sources_lower: &HashSet<String>,
        event: &TemporalEvent,
    ) -> Result<bool> {
        // Support for nested logical groups
        if let Some(logic) = rule_json.get("logic") {
            return Self::evaluate_logic_group(logic, haystack_lower, entities_lower, sources_lower, event);
        }

        // Legacy format: { "any": [...], "all": [...] }
        let any_conds = rule_json.get("any").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let all_conds = rule_json.get("all").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let any_pass = if any_conds.is_empty() {
            true
        } else {
            any_conds.iter().any(|c| {
                Self::condition_matches(c, haystack_lower, entities_lower, sources_lower, event)
                    .unwrap_or(false)
            })
        };

        let all_pass = all_conds
            .iter()
            .all(|c| {
                Self::condition_matches(c, haystack_lower, entities_lower, sources_lower, event)
                    .unwrap_or(false)
            });

        Ok(any_pass && all_pass)
    }

    fn evaluate_logic_group(
        logic: &Value,
        haystack_lower: &str,
        entities_lower: &HashSet<String>,
        sources_lower: &HashSet<String>,
        event: &TemporalEvent,
    ) -> Result<bool> {
        let op = logic.get("operator").and_then(|v| v.as_str()).unwrap_or("AND");
        
        match op {
            "AND" | "and" => {
                if let Some(conditions) = logic.get("conditions").and_then(|v| v.as_array()) {
                    for cond in conditions {
                        if !Self::evaluate_condition_or_group(cond, haystack_lower, entities_lower, sources_lower, event)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            "OR" | "or" => {
                if let Some(conditions) = logic.get("conditions").and_then(|v| v.as_array()) {
                    for cond in conditions {
                        if Self::evaluate_condition_or_group(cond, haystack_lower, entities_lower, sources_lower, event)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                } else {
                    Ok(false)
                }
            }
            "NOT" | "not" => {
                if let Some(condition) = logic.get("condition") {
                    Ok(!Self::evaluate_condition_or_group(condition, haystack_lower, entities_lower, sources_lower, event)?)
                } else {
                    Ok(false)
                }
            }
            "XOR" | "xor" => {
                if let Some(conditions) = logic.get("conditions").and_then(|v| v.as_array()) {
                    let mut true_count = 0;
                    for cond in conditions {
                        if Self::evaluate_condition_or_group(cond, haystack_lower, entities_lower, sources_lower, event)? {
                            true_count += 1;
                        }
                    }
                    Ok(true_count == 1)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn evaluate_condition_or_group(
        cond: &Value,
        haystack_lower: &str,
        entities_lower: &HashSet<String>,
        sources_lower: &HashSet<String>,
        event: &TemporalEvent,
    ) -> Result<bool> {
        // Check if it's a nested logic group
        if cond.get("logic").is_some() {
            return Self::evaluate_logic_group(cond, haystack_lower, entities_lower, sources_lower, event);
        }
        
        // Otherwise, it's a regular condition
        Self::condition_matches(cond, haystack_lower, entities_lower, sources_lower, event)
    }

    /// Enhanced condition matching with many more condition types
    pub fn condition_matches(
        cond: &Value,
        haystack_lower: &str,
        entities_lower: &HashSet<String>,
        sources_lower: &HashSet<String>,
        event: &TemporalEvent,
    ) -> Result<bool> {
        let t = cond.get("type").and_then(|v| v.as_str()).unwrap_or("");
        
        match t {
            // Text matching conditions
            "contains_keyword" => {
                cond.get("keyword")
                    .and_then(|v| v.as_str())
                    .map(|kw| haystack_lower.contains(&kw.to_lowercase()))
                    .ok_or_else(|| anyhow::anyhow!("Missing keyword"))
                    .map(|r| r)
            }
            "contains_regex" => {
                if let Some(pattern) = cond.get("pattern").and_then(|v| v.as_str()) {
                    let re = Regex::new(pattern)
                        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;
                    Ok(re.is_match(haystack_lower))
                } else {
                    Err(anyhow::anyhow!("Missing regex pattern"))
                }
            }
            "starts_with" => {
                cond.get("prefix")
                    .and_then(|v| v.as_str())
                    .map(|p| haystack_lower.starts_with(&p.to_lowercase()))
                    .ok_or_else(|| anyhow::anyhow!("Missing prefix"))
            }
            "ends_with" => {
                cond.get("suffix")
                    .and_then(|v| v.as_str())
                    .map(|s| haystack_lower.ends_with(&s.to_lowercase()))
                    .ok_or_else(|| anyhow::anyhow!("Missing suffix"))
            }
            
            // Entity conditions
            "mentions_entity" => {
                cond.get("entity")
                    .and_then(|v| v.as_str())
                    .map(|e| entities_lower.contains(&e.to_lowercase()))
                    .ok_or_else(|| anyhow::anyhow!("Missing entity"))
            }
            "mentions_any_entity" => {
                if let Some(entities) = cond.get("entities").and_then(|v| v.as_array()) {
                    Ok(entities.iter()
                        .filter_map(|v| v.as_str())
                        .any(|e| entities_lower.contains(&e.to_lowercase())))
                } else {
                    Err(anyhow::anyhow!("Missing entities array"))
                }
            }
            "mentions_all_entities" => {
                if let Some(entities) = cond.get("entities").and_then(|v| v.as_array()) {
                    let required: HashSet<String> = entities.iter()
                        .filter_map(|v| v.as_str())
                        .map(|e| e.to_lowercase())
                        .collect();
                    Ok(required.is_subset(entities_lower))
                } else {
                    Err(anyhow::anyhow!("Missing entities array"))
                }
            }
            "co_mention" => {
                let a = cond.get("entityA").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let b = cond.get("entityB").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                Ok(!a.is_empty() && !b.is_empty() && entities_lower.contains(&a) && entities_lower.contains(&b))
            }
            
            // Source conditions
            "source_in" => {
                if let Some(arr) = cond.get("sources").and_then(|v| v.as_array()) {
                    Ok(arr.iter()
                        .filter_map(|v| v.as_str())
                        .any(|s| sources_lower.contains(&s.to_lowercase())))
                } else {
                    Err(anyhow::anyhow!("Missing sources array"))
                }
            }
            "source_not_in" => {
                if let Some(arr) = cond.get("sources").and_then(|v| v.as_array()) {
                    Ok(!arr.iter()
                        .filter_map(|v| v.as_str())
                        .any(|s| sources_lower.contains(&s.to_lowercase())))
                } else {
                    Err(anyhow::anyhow!("Missing sources array"))
                }
            }
            
            // Score-based conditions with operators
            "sentiment" => {
                Self::compare_score(
                    event.sentiment_score,
                    cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">="),
                    cond.get("value").and_then(|v| v.as_f64())
                )
            }
            "volume" => {
                Self::compare_score(
                    event.volume_score,
                    cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">="),
                    cond.get("value").and_then(|v| v.as_f64())
                )
            }
            "novelty" => {
                Self::compare_score(
                    event.novelty_score,
                    cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">="),
                    cond.get("value").and_then(|v| v.as_f64())
                )
            }
            "severity" => {
                Self::compare_score(
                    event.severity,
                    cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">="),
                    cond.get("value").and_then(|v| v.as_f64())
                )
            }
            "confidence" => {
                Self::compare_score(
                    event.confidence,
                    cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">="),
                    cond.get("value").and_then(|v| v.as_f64())
                )
            }
            
            // Legacy aliases for backward compatibility
            "sentiment_below" => {
                cond.get("value")
                    .and_then(|v| v.as_f64())
                    .map(|v| Ok(event.sentiment_score <= v))
                    .unwrap_or(Err(anyhow::anyhow!("Missing value")))
            }
            "sentiment_above" => {
                cond.get("value")
                    .and_then(|v| v.as_f64())
                    .map(|v| Ok(event.sentiment_score >= v))
                    .unwrap_or(Err(anyhow::anyhow!("Missing value")))
            }
            "volume_spike" => {
                cond.get("value")
                    .and_then(|v| v.as_f64())
                    .map(|v| Ok(event.volume_score >= v))
                    .unwrap_or(Err(anyhow::anyhow!("Missing value")))
            }
            "novelty_above" => {
                cond.get("value")
                    .and_then(|v| v.as_f64())
                    .map(|v| Ok(event.novelty_score >= v))
                    .unwrap_or(Err(anyhow::anyhow!("Missing value")))
            }
            
            // Time-based conditions
            "time_of_day" => {
                if let (Some(start_hour), Some(end_hour)) = (
                    cond.get("start_hour").and_then(|v| v.as_i64()),
                    cond.get("end_hour").and_then(|v| v.as_i64()),
                ) {
                    let event_time = Utc.timestamp_opt(event.start_ts, 0)
                        .single()
                        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
                    let hour = event_time.naive_utc().hour() as i64;
                    Ok((hour >= start_hour && hour <= end_hour) || (start_hour > end_hour && (hour >= start_hour || hour <= end_hour)))
                } else {
                    Err(anyhow::anyhow!("Missing start_hour or end_hour"))
                }
            }
            "day_of_week" => {
                if let Some(days) = cond.get("days").and_then(|v| v.as_array()) {
                    let event_time = Utc.timestamp_opt(event.start_ts, 0)
                        .single()
                        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
                    let naive = event_time.naive_utc();
                    let weekday = naive.weekday().number_from_monday() as i64;
                    Ok(days.iter()
                        .filter_map(|v| v.as_i64())
                        .any(|d| d == weekday))
                } else {
                    Err(anyhow::anyhow!("Missing days array"))
                }
            }
            "date_range" => {
                if let (Some(start_ts), Some(end_ts)) = (
                    cond.get("start_ts").and_then(|v| v.as_i64()),
                    cond.get("end_ts").and_then(|v| v.as_i64()),
                ) {
                    Ok(event.start_ts >= start_ts && event.start_ts <= end_ts)
                } else {
                    Err(anyhow::anyhow!("Missing start_ts or end_ts"))
                }
            }
            
            // Event type conditions
            "event_type" => {
                cond.get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| Ok(event.event_type.to_lowercase() == t.to_lowercase()))
                    .unwrap_or(Err(anyhow::anyhow!("Missing type")))
            }
            "event_type_in" => {
                if let Some(types) = cond.get("types").and_then(|v| v.as_array()) {
                    Ok(types.iter()
                        .filter_map(|v| v.as_str())
                        .any(|t| event.event_type.to_lowercase() == t.to_lowercase()))
                } else {
                    Err(anyhow::anyhow!("Missing types array"))
                }
            }
            
            _ => {
                // Unknown condition type - log warning but don't fail
                eprintln!("Warning: Unknown condition type: {}", t);
                Ok(false)
            }
        }
    }

    fn compare_score(score: f64, operator: &str, threshold: Option<f64>) -> Result<bool> {
        let threshold = threshold.ok_or_else(|| anyhow::anyhow!("Missing threshold value"))?;
        
        match operator {
            ">" | "gt" => Ok(score > threshold),
            ">=" | "gte" => Ok(score >= threshold),
            "<" | "lt" => Ok(score < threshold),
            "<=" | "lte" => Ok(score <= threshold),
            "==" | "eq" | "equals" => Ok((score - threshold).abs() < 0.0001),
            "!=" | "ne" | "not_equals" => Ok((score - threshold).abs() >= 0.0001),
            _ => Err(anyhow::anyhow!("Unknown operator: {}", operator)),
        }
    }
}

