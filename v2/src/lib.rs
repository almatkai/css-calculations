//! Pure offline implementation of own pinned src/utils.ts. No OS effects.
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde_json::Value;

pub const FIELDS: [&str; 5] = [
    "min_font_size",
    "max_font_size",
    "min_vw_width",
    "max_vw_width",
    "pixels_per_rem",
];
pub const TITLES: [&str; 5] = [
    "Min Font Size",
    "Max Font Size",
    "Min Viewport Width",
    "Max Viewport Width",
    "Base Font Size",
];
pub const DEFAULTS: [&str; 5] = ["", "", "375", "1400", "16"];
/// Explicit resource bound, never truncates input. Applies to transport fields only.
pub const MAX_FIELD_BYTES: usize = 65_536;
fn js_space(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
fn decimal(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if b.get(i) == Some(&b'.') {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if start == i {
            return false;
        }
    }
    i == b.len()
}
pub fn number_string(s: &str) -> f64 {
    let s = s.trim_matches(js_space);
    match s {
        "" => return 0.0,
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => (),
    }
    let radix = if s.starts_with("0x") || s.starts_with("0X") {
        Some(16)
    } else if s.starts_with("0b") || s.starts_with("0B") {
        Some(2)
    } else if s.starts_with("0o") || s.starts_with("0O") {
        Some(8)
    } else {
        None
    };
    if let Some(radix) = radix {
        if s.len() == 2 || !s[2..].bytes().all(|b| (b as char).is_digit(radix)) {
            return f64::NAN;
        }
        return BigUint::parse_bytes(&s.as_bytes()[2..], radix)
            .and_then(|n| n.to_f64())
            .unwrap_or(f64::NAN);
    }
    if decimal(s) {
        s.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}
fn array_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => js_string(n.as_f64().unwrap_or(f64::NAN)),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => a.iter().map(array_string).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}
pub fn number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => u8::from(*b).into(),
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => number_string(s),
        Value::Array(_) => number_string(&array_string(v)),
        Value::Object(_) => f64::NAN,
    }
}
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(_) => {
            let n = number(v);
            n != 0.0 && !n.is_nan()
        }
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}
pub fn validate(key: &str, value: &Value, on_change: bool) -> Option<String> {
    let n = number(value);
    let suffix = if n.is_nan() {
        " is not a number!"
    } else if n < 0.0 {
        " must be grater than 0!"
    } else if if on_change { n == 0.0 } else { !truthy(value) } {
        " can not be empty!"
    } else {
        return None;
    };
    let label = FIELDS
        .iter()
        .position(|k| *k == key)
        .map(|i| {
            let title = TITLES[i].to_ascii_lowercase();
            format!("{}{}", title[..1].to_ascii_uppercase(), &title[1..])
        })
        .unwrap_or_else(|| "undefined".into());
    Some(label + suffix)
}
pub fn js_string(n: f64) -> String {
    ryu_js::Buffer::new().format(n).to_owned()
}
/// JS parseFloat(n.toFixed(4)), including exact binary ties and >=1e21 fallback.
pub fn rounded(n: f64) -> String {
    if !n.is_finite() || n.abs() >= 1e21 {
        return js_string(n);
    }
    let bits = n.abs().to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
    let shift = if exponent == 0 {
        -1074
    } else {
        exponent - 1023 - 52
    };
    let scaled = u128::from(mantissa) * 10_000;
    let integer = if shift >= 0 {
        scaled << shift
    } else {
        let right = (-shift) as u32;
        if right >= 128 {
            0
        } else {
            (scaled >> right) + ((scaled >> (right - 1)) & 1)
        }
    };
    let text = format!(
        "{}{}.{:04}",
        if n < 0.0 { "-" } else { "" },
        integer / 10_000,
        integer % 10_000
    );
    js_string(text.parse::<f64>().expect("bounded decimal"))
}
fn field_number(values: &Value, key: &str) -> f64 {
    values.get(key).map(number).unwrap_or(f64::NAN)
}
pub fn calculate(values: &Value, mode: &str) -> String {
    // Separate operations match JavaScript evaluation order; never algebraically simplify.
    let base = field_number(values, "pixels_per_rem");
    let min = field_number(values, "min_font_size");
    let max = field_number(values, "max_font_size");
    let min = if mode == "px" { min / base } else { min };
    let max = if mode == "px" { max / base } else { max };
    let low = field_number(values, "min_vw_width") / base;
    let high = field_number(values, "max_vw_width") / base;
    let slope = (max - min) / (high - low);
    let intercept = -low * slope + min;
    format!(
        "clamp({}rem, {}rem + {}vw, {}rem)",
        rounded(min),
        rounded(intercept),
        rounded(slope * 100.0),
        rounded(max)
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn preserved_submit_coercion_and_rounding() {
        assert!(validate("min_font_size", &json!("0"), false).is_none());
        assert!(validate("min_font_size", &json!("0"), true).is_some());
        assert_eq!(rounded(-0.00001), "0");
        assert_eq!(rounded(1e21), "1e+21");
        assert_eq!(rounded(f64::NAN), "NaN");
        assert_eq!(number_string("\u{feff}0x10\u{2029}"), 16.0);
        assert!(number_string("\u{0085}16").is_nan());
    }
}
