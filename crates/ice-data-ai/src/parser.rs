use serde_json::Value;
use tracing::warn;

/// Extracts and validates JSON from AI response text.
/// The response should be pure JSON (due to `response_format: json_object`),
/// but we fall back to extracting from markdown code blocks if needed.
pub fn parse_json_response(response: &str) -> Result<Value, String> {
    // Strip markdown code fences if present
    let cleaned = response
        .trim()
        .strip_prefix("```json")
        .or_else(|| response.trim().strip_prefix("```"))
        .map(|s| s.trim_end().strip_suffix("```").unwrap_or(s.trim_end()))
        .unwrap_or(response.trim());

    serde_json::from_str(cleaned).map_err(|e| format!("Failed to parse AI response as JSON: {e}"))
}

/// Validates that the parsed analysis has the required fields
/// and fills in defaults for missing optional fields.
pub fn normalize_analysis(parsed: &mut Value, analysis_type: &str) {
    let required_fields: &[&str] = match analysis_type {
        "full" => &["strengths", "weaknesses", "summary", "confidence_score"],
        "potential" => &["ceiling", "floor", "most_likely_outcome", "confidence_score"],
        "strengths" => &["elite_skills", "above_average_skills"],
        "comparison" => &["comparison_table", "recommendation", "confidence_score"],
        _ => &["strengths", "weaknesses"],
    };

    for field in required_fields {
        if !parsed.get(field).map_or(false, |v| !v.is_null()) {
            warn!(field, analysis_type, "Missing required field in AI response");
        }
    }

    // Ensure arrays exist
    for arr_field in &["strengths", "weaknesses", "elite_skills", "above_average_skills",
                       "development_areas", "comparables", "key_development_areas"]
    {
        if parsed.get(*arr_field).map_or(true, |v| !v.is_array()) {
            parsed[*arr_field] = Value::Array(vec![]);
        }
    }

    // Ensure confidence_score exists
    if parsed.get("confidence_score").map_or(true, |v| !v.is_f64()) {
        parsed["confidence_score"] = Value::Number(serde_json::Number::from_f64(0.7).unwrap_or(serde_json::Number::from(0)));
    }
}

/// Merges multiple analysis results into one comprehensive report.
pub fn merge_analyses(analyses: Vec<Value>) -> Value {
    let mut merged = serde_json::json!({
        "strengths": [],
        "weaknesses": [],
        "development_areas": [],
        "confidence_score": 0.0,
    });

    let mut all_strengths: Vec<Value> = vec![];
    let mut all_weaknesses: Vec<Value> = vec![];
    let mut total_confidence = 0.0_f64;
    let count = analyses.len().max(1);

    for analysis in &analyses {
        if let Some(s) = analysis.get("strengths").and_then(|v| v.as_array()) {
            all_strengths.extend(s.clone());
        }
        if let Some(s) = analysis.get("weaknesses").and_then(|v| v.as_array()) {
            all_weaknesses.extend(s.clone());
        }
        if let Some(s) = analysis.get("development_areas").and_then(|v| v.as_array()) {
            if let Some(da) = merged["development_areas"].as_array_mut() {
                da.extend(s.clone());
            }
        }
        if let Some(c) = analysis.get("confidence_score").and_then(|v| v.as_f64()) {
            total_confidence += c;
        }
    }

    merged["strengths"] = Value::Array(all_strengths);
    merged["weaknesses"] = Value::Array(all_weaknesses);
    merged["confidence_score"] = Value::Number(
        serde_json::Number::from_f64(total_confidence / count as f64)
            .unwrap_or(serde_json::Number::from(0)),
    );

    // Carry forward full_report and summary if present
    for key in &["full_report", "summary", "recommendation", "key_insight",
                  "ceiling", "floor", "most_likely_outcome"]
    {
        if let Some(val) = analyses.iter().find_map(|a| a.get(*key)).cloned() {
            merged[key] = val;
        }
    }

    merged
}
