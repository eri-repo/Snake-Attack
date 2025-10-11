pub mod Structs {
    use starknet::ContractAddress;
   

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct UserInfo {
        // pub address: ContractAddress,
        pub username: felt252,
        // pub position: u128,
        // pub is_registered: bool,
        pub highest_score: u256,
        pub updated: bool,
        // pub registered_at: u64,
    }

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct BoardInfo {
        pub player: ContractAddress,
        pub username: felt252,
        pub score: u256,
    }

    #[derive(Copy, Drop, Serde, PartialEq, starknet::Store)]
    pub enum ChallengeStatus {
        #[default]
        CREATED,
        ACCEPTED,
        REJECTED,
        COUNTERED: felt252,
    }
    #[derive(Copy, Drop, Serde, PartialEq, starknet::Store)]
    pub enum CounterStatus {
        #[default]
        CREATED,
        ACCEPTED,
        REJECTED,
    }

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct ChallengeDetails {
        pub address: ContractAddress,
        pub points_locked: u256,
        pub score: u256,
    }

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct Challenge {
        pub challenge_id: felt252,
        pub challenger: ChallengeDetails,
        pub challenged: ChallengeDetails,
        pub stake_amount: u256,
        pub winner: ContractAddress,
        pub number_of_games: u16,
        pub points_to_deduct: u256,
        pub status: ChallengeStatus,
        pub completed: bool,
        pub created_at: u64,
    }

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct CounterChallenge {
        pub counter_id: felt252,
        pub challenge_id: felt252,
        pub stake_amount: u256,
        pub number_of_games: u16,
        pub points_to_deduct: u256,
        pub status: CounterStatus,
    }

    #[derive(Drop, Serde, Clone, PartialEq, starknet::Store)]
    pub struct PlayerDetails {
        pub address: ContractAddress,
        pub score: u256,
    }

    #[derive(Drop, Serde, Clone, PartialEq)]
    pub struct MultiPlayerChallenge {
        pub challenge_id: felt252,
        pub players: Span<PlayerDetails>,
        pub stake_amount: u256,
        pub number_of_games: u16,
        pub status: CounterStatus,
        pub created_at: u64,
    }
}

