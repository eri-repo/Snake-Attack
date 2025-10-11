pub mod Events {
    use starknet::ContractAddress;

    #[derive(Drop, starknet::Event)]
    pub struct ContractCreated {
        #[key]
        pub contract_address: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PlayerRegistered {
        #[key]
        pub player_address: ContractAddress,
        #[key]
        pub username: felt252,
    }

    #[derive(Drop, starknet::Event)]
    pub struct UsernameUpdted {
        #[key]
        pub old_username: felt252,
        pub new_username: felt252,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ChallengeCreated {
        #[key]
        pub challenge_id: felt252,
        #[key]
        pub challenged: ContractAddress,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct ScoreUpdated {
        #[key]
        pub score: u256,
        // #[key]
        // pub current_score: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct CounterUpdateChallenge {
        #[key]
        pub challenge_id: felt252
    }

    #[derive(Drop, starknet::Event)]
    pub struct ChallengeUpdated {
        #[key]
        pub challenge_id: felt252,
        #[key]
        pub winner: ContractAddress
    }

    #[derive(Drop, starknet::Event)]
    pub struct ChallengeAccepted {
        #[key]
        pub challenge_id: felt252
    }
}

