fn main() {
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        use nimiq_serde::Deserialize as _;
        use nimiq_primitives::coin::Coin;
        let _ = Coin::deserialize_from_vec(data);
    })
}
