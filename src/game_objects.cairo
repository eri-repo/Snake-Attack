pub mod GameObjects {   

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct UserInfo {
        pub username: felt252,
        pub highest_score: u256,
        pub updated: bool,
    }
}

