/// npm-to-yarn's `parse`: split at spaces outside quotes. A backslash keeps a quote from
/// opening or closing, but not a space from splitting, and stays in the argument.
pub(crate) fn parse_args(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut part = String::new();
    for character in command.chars() {
        if character == '\\' {
            part.push(character);
            escaped = true;
            continue;
        }
        if character == ' ' && quote.is_none() {
            args.push(std::mem::take(&mut part));
        } else if !escaped && matches!(character, '"' | '\'') {
            part.push(character);
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
        } else {
            part.push(character);
        }
        escaped = false;
    }
    args.push(part);
    args
}
