use crate::utils::tokenizer::driver::Tokenizer;

/// Track where the runs of lines that definition terms come from start, once the
/// containers of a line are done, as micromark's extension reads them back from the
/// document's events.
pub(crate) fn track_def_list(tokenizer: &mut Tokenizer, lazy: bool) {
    let line_events = has_line_events(tokenizer);
    let start = tokenizer.point.index;
    let track = &mut tokenizer.tokenize_state.def_list;
    if track.line_boundary {
        track.run_start = start as usize;
    }
    if line_events || track.line_exits {
        track.pure_start = start as usize;
    }
    track.lazy_before = track.lazy_start.is_some();
    if !lazy {
        track.lazy_start = None;
    } else if track.lazy_start.is_none() {
        track.lazy_start = Some(start as usize);
    }
    track.line_start = start as usize;
    track.line_boundary = false;
    track.line_exits = false;
}

/// Whether the containers of this line put events on it so far.
pub(crate) fn has_line_events(tokenizer: &Tokenizer) -> bool {
    match tokenizer.tokenize_state.document_data_index {
        Some(index) => tokenizer.events.len() > index + 2,
        None => !tokenizer.events.is_empty(),
    }
}
