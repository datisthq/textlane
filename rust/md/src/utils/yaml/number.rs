use serde_json::Value;

/// JavaScript's `String(number)`: the shortest digits, in fixed notation from 1e-6 up to
/// 1e21, else exponential.
pub(crate) fn format_number(number: f64) -> String {
    if number.is_nan() {
        return "NaN".into();
    }
    if number.is_infinite() {
        return if number > 0.0 {
            "Infinity"
        } else {
            "-Infinity"
        }
        .into();
    }
    if number == 0.0 {
        return "0".into();
    }
    let sign = if number < 0.0 { "-" } else { "" };
    let scientific = format!("{:e}", number.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let digits = mantissa.replace('.', "");
    let exponent = exponent.parse::<i64>().unwrap_or(0) + 1;
    let length = digits.len() as i64;
    let body = if length <= exponent && exponent <= 21 {
        format!("{digits}{}", "0".repeat((exponent - length) as usize))
    } else if 0 < exponent && exponent <= 21 {
        let (whole, fraction) = digits.split_at(exponent as usize);
        format!("{whole}.{fraction}")
    } else if -6 < exponent && exponent <= 0 {
        format!("0.{}{digits}", "0".repeat((-exponent) as usize))
    } else {
        let power = exponent - 1;
        let power_sign = if power < 0 { "-" } else { "+" };
        let (first, rest) = digits.split_at(1);
        let dot = if rest.is_empty() { "" } else { "." };
        format!("{first}{dot}{rest}e{power_sign}{}", power.abs())
    };
    format!("{sign}{body}")
}

/// A JavaScript number as JSON reads it back after `JSON.stringify`: infinities and NaN
/// become null, and integers written without an exponent stay integers.
pub(crate) fn number_value(number: f64) -> Value {
    if !number.is_finite() {
        return Value::Null;
    }
    serde_json::from_str(&format_number(number)).unwrap_or(Value::Null)
}

/// JavaScript's `parseInt` over digits already checked to be in `radix`.
pub(crate) fn parse_digits(digits: &str, radix: u32) -> f64 {
    if radix == 10 {
        return digits.parse().unwrap_or(f64::NAN);
    }
    if let Ok(whole) = u128::from_str_radix(digits, radix) {
        return whole as f64;
    }
    digits.chars().fold(0.0, |total, digit| {
        total * f64::from(radix) + f64::from(digit.to_digit(radix).unwrap_or(0))
    })
}
