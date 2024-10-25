fn main() {
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        use nimiq_serde::Deserialize as _;
        use nimiq_keys::KeyPair;
        let _ = KeyPair::deserialize_from_vec(data);
    })
}
