use super::*;
use crate::{Processor, read_retained_memory};

// One test, since the setting is shared by every thread the test binary runs
#[test]
fn set_retained_memory_decides_what_a_thread_keeps() -> Result<(), crate::Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    let previous = read_retained_memory();
    assert_eq!(previous, 256 << 20);

    set_retained_memory(0);
    assert_eq!(read_retained_memory(), 0);
    processor.process("# a")?;
    assert_eq!(crate::actions::allocator::reuse::read_spare_bytes(), None);

    set_retained_memory(previous);
    processor.process("# a")?;
    assert!(crate::actions::allocator::reuse::read_spare_bytes().is_some());
    Ok(())
}
