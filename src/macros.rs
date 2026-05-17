/// convert string literals into byte slices (`&[&[u8]]`).
#[macro_export]
macro_rules! byte_slices {
    ($($lit:literal),* $(,)?) => {
        &[$($lit.as_bytes()),*]
    };
}
