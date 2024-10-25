fn main() { 
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        use nimiq_serde::{Deserialize, Serialize};
        use nimiq_account::StakingContract;
        let res = StakingContract::deserialize_from_vec(data); // err I think is of type DeserializeError

        // Now check if contr exists. If it does (aka. the original data was a valid staking contract) then try to serialize it back to a vector, then check if the original vector and the new vector are the same, if they aren't then there is a bug in the parsing logic.

        // The existance of error implies that contr does not exist.

        match res {
            Ok(v) => {
                // First calculate the minimum of the two.
                let otherdata = StakingContract::serialize_to_vec(&v);

                /*
                let buf = (0..64).collect::<Vec<u8>>();
                let z: &[u8; 64] = &(&buf[..]).try_into().unwrap();
                */

                assert!((otherdata.len() <= data.len()), "The size of the serialized version was bigger than the original vector! This shouldn't happen!");
                //data.resize((otherdata.len()), 0);
                
                // let comparison_bullshit = data.collect::<Vec<u8>>(); // Collect the stuff

                // arg[..30].try_into().unwrap()

                let bullshit: &[u8] = data[..(otherdata.len())].try_into().unwrap(); // Yuck!!!

                assert_eq!(bullshit, otherdata);
            },
            Err(e) => {
                // println!("Error thing!!!\n");
                return;
            },
        }

        /*
        if (err == None) {
            // Contr exists
            assert!(contr, "contr didn't exist, even though err == None!!!!"); // Debug.
            // Now try to serialize to vec
            let serialized = StakingContract::serialize_to_vec(contr);
            // Now check if they are equal:
            assert_eq!(data, serialized);
        }
        */


    })
}

/*
    let contract_2a =
        StakingContract::deserialize_from_vec(&contract_2.serialize_to_vec()).unwrap();

    assert_eq!(contract_2, contract_2a);
*/
