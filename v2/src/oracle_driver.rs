use css_calculations_native_v2::{calculate, validate, FIELDS};
use serde_json::{json, Value};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin");
        let case: Value = serde_json::from_str(&line).expect("JSON case");
        let values = &case["values"];
        let errors: Vec<_> = values
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(k, v)| validate(k, v, false).map(|e| json!({"key":k,"value":e})))
            .collect();
        let change: Vec<_> = FIELDS
            .iter()
            .map(|k| json!(validate(k, &values[k], true)))
            .collect();
        println!(
            "{}",
            json!({"css":calculate(values,case["mode"].as_str().unwrap()),"errors":errors,"change":change})
        );
    }
}
