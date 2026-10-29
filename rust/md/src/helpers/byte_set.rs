/// A set of bytes, one bit each, so a lookup costs the same however many there are.
pub(crate) type ByteSet = [u64; 4];

/// The set of `bytes`.
pub(crate) const fn to_byte_set(bytes: &[u8]) -> ByteSet {
    let mut set = [0; 4];
    let mut index = 0;
    while index < bytes.len() {
        set = add_byte(set, bytes[index]);
        index += 1;
    }
    set
}

/// `set` with `byte` in it.
pub(crate) const fn add_byte(mut set: ByteSet, byte: u8) -> ByteSet {
    set[(byte >> 6) as usize] |= 1 << (byte & 63);
    set
}

/// Whether `byte` is in `set`.
pub(crate) fn has_byte(set: &ByteSet, byte: u8) -> bool {
    set[usize::from(byte >> 6)] & (1 << (byte & 63)) != 0
}
