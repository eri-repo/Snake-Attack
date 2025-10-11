use starknet::ContractAddress;
// use crate::structs::Structs::BoardInfo;
// use crate::structs::Structs::ChallengeDetails;


#[starknet::interface]
pub trait IStarknake<TContractState> {
    fn player_registers(ref self: TContractState, player_address: ContractAddress, username: felt252);
    fn player_update_username(ref self: TContractState, username: felt252);
    fn update_player_score(ref self: TContractState, address: ContractAddress, current_score: u256);
    // fn is_registered(self: @TContractState, user_addess: ContractAddress) -> bool;
    // fn get_leaderboard(self: @TContractState) -> Array<BoardInfo>;
    // fn challenge_player(
    //     ref self: TContractState,
    //     challenged: ContractAddress,
    //     stake_amount: u256,
    //     number_of_games: u16,
    //     points_to_deduct: u256,
    // );

    // fn counter_update_challenge(
    //     ref self: TContractState,
    //     challenge_id: felt252,
    //     stake_amount: u256,
    //     number_of_games: u16,
    //     points_to_deduct: u256,
    // );

    // fn settle_challenge(
    //     ref self: TContractState,
    //     challenge_id: felt252,
    //     challenger: ChallengeDetails,
    //     challenged: ChallengeDetails
    // );

    // fn player_accept_challenge(
    //     ref self: TContractState,
    //     challenge_id: felt252,
    //     player_address: ContractAddress,
    // );
}


#[starknet::interface]
pub trait ISTRK<TContractState> {
    fn balance_of(self: @TContractState, account: ContractAddress) -> u256;
    fn transfer(ref self: TContractState, recipient: ContractAddress, amount: u256) -> bool;
}
