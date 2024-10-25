fn main() {
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        use nimiq_serde::Deserialize as _;
        use nimiq_account::{
            HashedTimeLockedContract
        }; 
        let _ = HashedTimeLockedContract::deserialize_from_vec(data);
    })
}