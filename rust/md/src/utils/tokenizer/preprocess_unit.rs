use super::{MAX_INPUT, check_length};

#[test]
fn caps_input_where_tab_columns_still_fit_in_u32() {
    assert!(
        MAX_INPUT
            .checked_mul(4)
            .is_some_and(|columns| columns <= u32::MAX as usize)
    );
    assert!(check_length(MAX_INPUT).is_ok());
    let error = check_length(MAX_INPUT + 1).err();
    assert_eq!(
        error.map(|error| error.code),
        Some("input-too-long".to_string())
    );
}
