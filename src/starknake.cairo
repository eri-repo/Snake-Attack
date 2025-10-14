#[starknet::contract]
pub mod Starknake {
    use core::array::ArrayTrait;
    use core::num::traits::Zero;
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, StoragePathEntry, StoragePointerReadAccess, StoragePointerWriteAccess,
        
    };
    use starknet::{ContractAddress, get_caller_address, get_contract_address};
    use crate::errors::Errors::*;
    use crate::events::Events::*;
    use crate::interfaces::*;
    use crate::game_objects::GameObjects::*;
    use crate::utilities::Utilities::*;


    //events
    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ContractCreated: ContractCreated,
        PlayerRegistered: PlayerRegistered,
        UsernameUpdted: UsernameUpdted,
        ScoreUpdated: ScoreUpdated,
    }


    #[storage]
    struct Storage {
        owner: ContractAddress,
        users: Map<ContractAddress, UserInfo>,
        usernames: Map<felt252, ContractAddress>,
    }


    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        address_zero_check(owner);

        self.owner.write(owner);

        self.emit(ContractCreated { contract_address: get_contract_address() });
    }


    #[abi(embed_v0)]
    impl Starknake of IStarknake<ContractState> {
        fn player_registers(
            ref self: ContractState, player_address: ContractAddress, username: felt252,
        ) {
            // assert(
            //     get_caller_address() == self.owner.read(), ONLY_OWNER,
            // ); //only the owner can register players

            // assert he is not registered to avoid duplicate registration
            assert(self.users.entry(player_address).read().username == 0, ALREADY_REGISTERED);
            // to make sure usernames are unique
            assert(self.usernames.entry(username).read().is_zero(), USERNAME_TAKEN);

            let player = UserInfo { username, highest_score: 0_u256, updated: false };

            self.users.entry(player_address).write(player);
            self.usernames.entry(username).write(player_address);

            self.emit(PlayerRegistered { player_address: player_address, username: username });
        }


        //player only get to update their username once
        //usually, at registration, the frontend will generate a random username for the player
        //so the player does not need to provide a username at registration
        //so a simple wallet connection registers the player if he's not already registered
        //and the player can update this username later
        fn player_update_username(ref self: ContractState, username: felt252) {
            let caller = get_caller_address();
            address_zero_check(caller);

            //check if he's registered
            assert(self.users.entry(caller).read().username != 0, NOT_PLAYER);

            // You can only updated username once after registration
            // on the frontend, if you dont supply a username, we will generate a random one for you
            // at regisrtration
            //this will be checked on the backend
            assert(!self.users.entry(caller).read().updated, UPDATE_LIMIT);

            let mut found_player = self.users.entry(caller).read();
            let old_username = found_player.username;

            found_player.username = username; //update username
            found_player.updated = true; //set updated to true so player cannot update again

            self.users.entry(caller).write(found_player);

            self.emit(UsernameUpdted { old_username, new_username: username });
        }

        //this is used to update the player's score when player plays single game
        //the single game score is updated on-chain when the player exits the game
        //and the accumulated score is updated on-chain
        //we are making this call on behalf of the player so we are paying the gas fee
        fn update_player_score(
            ref self: ContractState, address: ContractAddress, current_score: u256,
        ) {
            assert(self.users.entry(address).read().username != 0, NOT_PLAYER);

            let mut player = self.users.entry(address).read();

            player.highest_score = player.highest_score + current_score;

            self.users.entry(address).write(player);

            self.emit(ScoreUpdated { score: current_score });
        }
    }
}
