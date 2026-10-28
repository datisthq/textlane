use textlane_core::mdast::Math;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::measure_longest_streak;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::Serializer;

/// Write math (flow), with a fence longer than any run of dollars inside.
pub(crate) fn stringify_math(state: &mut Serializer, math: &Math) {
    let raw = math.value;
    let sequence = "$".repeat((measure_longest_streak(raw, '$') + 1).max(2));
    state.enter(Construct::MathFlow);
    let mut value = sequence.clone();
    if let Some(meta) = math.meta.filter(|meta| !meta.is_empty()) {
        state.enter(Construct::MathFlowMeta);
        let config = SafeConfig {
            before: &value,
            after: "\n",
            encode: &['$'],
            ..SafeConfig::default()
        };
        let safe = state.safe(meta, &config);
        value.push_str(&safe);
        state.exit();
    }
    value.push('\n');
    if !raw.is_empty() {
        value.push_str(raw);
        value.push('\n');
    }
    value.push_str(&sequence);
    state.exit();
    state.write(&value);
}
