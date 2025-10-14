use starknet::ContractAddress;
// use crate::structs::Structs::BoardInfo;
// use crate::structs::Structs::ChallengeDetails;


#[starknet::interface]
pub trait IStarknake<TContractState> {
    fn player_registers(ref self: TContractState, player_address: ContractAddress, username: felt252);
    fn player_update_username(ref self: TContractState, username: felt252);
    fn update_player_score(ref self: TContractState, address: ContractAddress, current_score: u256);
}