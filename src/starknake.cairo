#[starknet::contract]
pub mod Starknake {
    use core::array::ArrayTrait;
    use core::hash::{HashStateExTrait, HashStateTrait};
    use core::num::traits::Zero;
    use core::poseidon::PoseidonTrait;
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, MutableVecTrait, StoragePathEntry, StoragePointerReadAccess, StoragePointerWriteAccess,
        Vec, VecTrait,
    };
    use starknet::{ContractAddress, get_block_timestamp, get_caller_address, get_contract_address};
    use crate::errors::Errors::*;
    use crate::events::Events::*;
    use crate::interfaces::{*, ISTRKDispatcher, ISTRKDispatcherTrait};
    use crate::structs::Structs::*;
    use crate::utilities::Utilities::*;


    //events
    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ContractCreated: ContractCreated,
        PlayerRegistered: PlayerRegistered,
        UsernameUpdted: UsernameUpdted,
        ChallengeCreated: ChallengeCreated,
        ScoreUpdated: ScoreUpdated,
        CounterUpdateChallenge: CounterUpdateChallenge,
        ChallengeUpdated: ChallengeUpdated,
        ChallengeAccepted: ChallengeAccepted,
    }

    const STRK_TOKEN_ADDRESS: ContractAddress =
        0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
        .try_into()
        .unwrap();


    #[storage]
    struct Storage {
        owner: ContractAddress,
        // users_info: Map<ContractAddress, felt252>,
        users: Map<ContractAddress, UserInfo>,
        //global_position: u128, //this will be given to the newest player that registers
        usernames: Map<felt252, ContractAddress>,
        // leaderboard: Vec<BoardInfo>,
        // challenges: Map<felt252, Challenge>,
        // multi: Map<felt252, MultiPlayerChallenge>,
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


        //THIS IS DEFINITELY GOING ON OFF-CHAIN DATABASE
        //this will get the leaderboard
        //on the frontend, the sorting will be done and the first 20 on the leaderboard will be
        //shown this is to avoid expensive transaction of sorting onchain
        //THIS WILL BE DONE ON THE BACKTEND
        // fn get_leaderboard(self: @ContractState) -> Array<BoardInfo> {
        //     let mut leaders = array![];

        //     for i in 0..self.leaderboard.len() {
        //         leaders.append(self.leaderboard.at(i).read());
        //     }
        //     leaders
        // }

        //TO THE FRONTEND AND SAVED IN DATABASE => ONLY CREATE CHALLENGE FUNCTION WILL BE ON-CHAIN
        //FOR CONCLUDED CHALLENGES Player challenges another player
        //the challenged can either accept the challenge or counter challenge the challenger with
        //his own challenge condition if the challenged accepts the challenger's condition, the
        //challenge goes on but if the challenged counter challenge the challenger, the challenger
        //can accept the counter challenge (the counter challenge info will be used to update the
        //original challenge) or updates the original challenge and send it back to the challenged..
        //this back and forth could continue until there's an agreement.
        //only the challenge is saved on-chain, the counter challenge is not saved on-chain.
        // fn challenge_player(
        //     ref self: ContractState,
        //     challenged: ContractAddress,
        //     stake_amount: u256,
        //     number_of_games: u16,
        //     points_to_deduct: u256,
        // ) {
        //     let caller = get_caller_address();
        //     address_zero_check(caller);

        //     // check if the caller is registered
        //     assert(self.users.entry(caller).read().username != 0, NOT_PLAYER);
        //     // check if the challenged player is registered
        //     assert(self.users.entry(challenged).read().username != 0, NOT_PLAYER);

        //     // check if the caller is not challenging himself
        //     assert(caller != challenged, CANNOT_CHALLENGE_SELF);

        //     let time = get_block_timestamp();

        //     let id = PoseidonTrait::new()
        //         .update_with(challenged)
        //         .update_with(caller)
        //         .update_with(time)
        //         .finalize();

        //     // create a challenge object
        //     let challenge = Challenge {
        //         challenge_id: id, // this will be unique for each challenge
        //         challenger: ChallengeDetails {
        //             address: caller, score: 0_u256, points_locked: 0_u256,
        //         },
        //         challenged: ChallengeDetails {
        //             address: challenged, score: 0_u256, points_locked: 0_u256,
        //         },
        //         winner: 0x0.try_into().unwrap(), //address zero
        //         stake_amount,
        //         number_of_games,
        //         points_to_deduct,
        //         status: ChallengeStatus::CREATED,
        //         completed: false,
        //         created_at: time,
        //     };

        //     // store the challenge in the storage
        //     self.challenges.entry(id).write(challenge);

        //     self.emit(ChallengeCreated { challenge_id: id, challenged });
        // }

        //==========
        // fn create_challenge(
        //     ref self: ContractState,
        //     challenged: ContractAddress,
        //     stake_amount: u256,
        //     number_of_games: u16,
        //     points_to_deduct: u256,
        // ) {
        //     let caller = get_caller_address();
        //     address_zero_check(caller);

        //     // check if the caller is registered
        //     assert(self.users.entry(caller).read().username != 0, NOT_PLAYER);
        //     // check if the challenged player is registered
        //     assert(self.users.entry(challenged).read().username != 0, NOT_PLAYER);

        //     // check if the caller is not challenging himself
        //     assert(caller != challenged, CANNOT_CHALLENGE_SELF);

        //     let time = get_block_timestamp();

        //     let id = PoseidonTrait::new()
        //         .update_with(challenged)
        //         .update_with(caller)
        //         .update_with(time)
        //         .finalize();

        //     // create a challenge object
        //     let challenge = Challenge {
        //         challenge_id: id, // this will be unique for each challenge
        //         challenger: ChallengeDetails {
        //             address: caller, score: 0_u256, points_locked: 0_u256,
        //         },
        //         challenged: ChallengeDetails {
        //             address: challenged, score: 0_u256, points_locked: 0_u256,
        //         },
        //         winner: 0x0.try_into().unwrap(), //address zero
        //         stake_amount,
        //         number_of_games,
        //         points_to_deduct,
        //         status: ChallengeStatus::CREATED,
        //         completed: false,
        //         created_at: time,
        //     };

        //     // store the challenge in the storage
        //     self.challenges.entry(id).write(challenge);

        //     self.emit(ChallengeCreated { challenge_id: id, challenged });
        // }

        //==========

        //THIS WILL BE DONE ON THE OFF-CHAIN WITH CHALLENGE ON THE DATABASE
        //this function is called when the challenged accepts the challenger's challenge without
        //counter challenging
        // the function is called to update the challenge status to ACCEPTED
        // and the deducted points are locked
        //on the frontend, the players will stake STRK tokens during this period so they can play
        //the game
        // fn player_accept_challenge(
        //     ref self: ContractState, challenge_id: felt252, player_address: ContractAddress,
        // ) {
        //     let caller = get_caller_address();
        //     address_zero_check(caller);

        //     // check if the caller is registered
        //     assert(self.users.entry(caller).read().is_registered, NOT_PLAYER);

        //     // retrieve the challenge from storage
        //     let mut challenge = self.challenges.entry(challenge_id).read();

        //     // check if the player is the challenged player
        //     assert(challenge.challenged.address == player_address, NOT_OWNER);

        //     // check if the challenge exists and is not completed
        //     assert(!challenge.completed, 'CHALLENGE ALREADY COMPLETED');

        //     //to deduct points, the challenger must have enough points locked
        //     assert(
        //         self.users.entry(caller).read().highest_score >= challenge.points_to_deduct,
        //         INSUFFICIENT_POINTS,
        //     );

        //     self
        //         .lock_point(
        //             challenge_id,
        //             challenge.challenger.address,
        //             challenge.challenged.address,
        //             challenge.points_to_deduct,
        //         );

        //     // update the challenge status to accepted
        //     challenge.status = ChallengeStatus::ACCEPTED;

        //     self.challenges.entry(challenge_id).write(challenge);

        //     self.emit(ChallengeAccepted { challenge_id });
        // }

        //THIS IS DEFINITELY GOING TO THE FRONTEND AND ON OFF-CHAIN DATABASE
        //this will be used when the challenged counters the challenge and the challenger
        //accepts the counter challenge the counter challenge info is used to update the original
        //challenge this is to avoid the original challenger having to create a new challenge.
        //NB: The counter challenge is not saved on-chain but on a database on the backend
        //so that the original challenge can be updated with the counter challenge info
        // fn counter_update_challenge(
        //     ref self: ContractState,
        //     challenge_id: felt252,
        //     stake_amount: u256,
        //     number_of_games: u16,
        //     points_to_deduct: u256,
        // ) {
        //     let caller = get_caller_address();
        //     address_zero_check(caller);

        //     // check if the caller is registered
        //     assert(self.users.entry(caller).read().is_registered, NOT_PLAYER);

        //     assert(
        //         self.challenges.entry(challenge_id).read().challenger.address == caller, NOT_OWNER,
        //     );

        //     // retrieve the challenge from storage
        //     let mut challenge = self.challenges.entry(challenge_id).read();

        //     // check if the challenge exists and is not completed
        //     assert(!challenge.completed, 'CHALLENGE ALREADY COMPLETED');

        //     //to deduct points, the challenger must have enough points locked
        //     assert(
        //         self.users.entry(caller).read().highest_score >= points_to_deduct,
        //         INSUFFICIENT_POINTS,
        //     );

        //     challenge.stake_amount = stake_amount;
        //     challenge.number_of_games = number_of_games;
        //     challenge.points_to_deduct = points_to_deduct;
        //     challenge.status = ChallengeStatus::ACCEPTED;

        //     self.challenges.entry(challenge_id).write(challenge);

        //     self.emit(CounterUpdateChallenge { challenge_id });
        // }

        //THIS WILL BE DONE ON THE FRONTEND AND WHEN THE CHALLENGE IS COMPLETED/SETTLED, WE WILL
        //SEND THE COMPLETED CHALLENGE ON-CHAIN i will still need to work on this function as the
        //caller will have to be us and not the players
        // the owner will be the caller of this function to settle the challenge and this will be
        // set up automatically
        //at the end of the game, there will be a check if the game is completed and on the
        //frontend, the winner will be determined and once a winner is determined, this function
        //will be called and the winner will be determined on-cahin also
        // fn settle_challenge( //TODO: WORK IS NOT DONE HERE
        //     ref self: ContractState,
        //     challenge_id: felt252,
        //     challenger: ChallengeDetails,
        //     challenged: ChallengeDetails,
        // ) {
        //     let caller = get_caller_address();
        //     address_zero_check(caller); //this check will still be only owner

        //     // retrieve the challenge from storage
        //     let mut challenge = self.challenges.entry(challenge_id).read();

        //     // check if the challenge exists and is not completed
        //     assert(!challenge.completed, 'CHALLENGE ALREADY COMPLETED');

        //     let mut winner = 0x0.try_into().unwrap();

        //     if challenger.score > challenged.score {
        //         winner = challenger.address;
        //         self
        //             .update_points(
        //                 challenger.address, challenged.address, challenge.points_to_deduct,
        //             );
        //     } else if challenged.score > challenger.score {
        //         winner = challenged.address;
        //         self
        //             .update_points(
        //                 challenged.address, challenger.address, challenge.points_to_deduct,
        //             );
        //     } else {
        //         winner = 0x0.try_into().unwrap();
        //     }

        //     // update the challenge details
        //     challenge.challenger.score = challenger.score;
        //     challenge.challenged.score = challenged.score;
        //     challenge.winner = winner;
        //     challenge.completed = true; // mark the challenge as completed

        //     if winner != 0x0.try_into().unwrap() { //this is to pay winner
        //         //contract transfer the stake amount to the winner
        //         let strk_contract = ISTRKDispatcher { contract_address: STRK_TOKEN_ADDRESS };
        //         strk_contract.transfer(winner, challenge.stake_amount);
        //     }

        //     self.challenges.entry(challenge_id).write(challenge);

        //     self.emit(ChallengeUpdated { challenge_id, winner });
        // }
    }

    // #[generate_trait] //privates
    // impl Utility of IUtility {
    //     fn is_registered(self: @ContractState, user_addess: ContractAddress) -> bool {
    //         self.users.entry(user_addess).read().is_registered
    //     }

    //     fn update_points(
    //         ref self: ContractState,
    //         address1: ContractAddress,
    //         address2: ContractAddress,
    //         updated_points: u256,
    //     ) {
    //         let mut player1 = self.users.entry(address1).read();
    //         let mut player2 = self.users.entry(address2).read();

    //         let index1 = (player1.position - 1).try_into().unwrap_or(0);
    //         let index2 = (player2.position - 1).try_into().unwrap_or(0);

    //         let mut board1 = self.leaderboard.get(index1).unwrap().read();
    //         let mut board2 = self.leaderboard.get(index2).unwrap().read();

    //         board1.score = board1.score + updated_points;
    //         board2.score = board2.score - updated_points;

    //         player1.highest_score = board1.score;
    //         player2.highest_score = board2.score;

    //         self.leaderboard.get(index1).unwrap().write(board1);
    //         self.leaderboard.get(index2).unwrap().write(board2);

    //         self.users.entry(address1).write(player1);
    //         self.users.entry(address2).write(player2);
    //     }

    //     fn lock_point(
    //         ref self: ContractState,
    //         challenge_id: felt252,
    //         address1: ContractAddress,
    //         address2: ContractAddress,
    //         updated_points: u256,
    //     ) {
    //         let mut player1 = self.users.entry(address1).read();
    //         let mut player2 = self.users.entry(address2).read();

    //         let index1 = (player1.position - 1).try_into().unwrap_or(0);
    //         let index2 = (player2.position - 1).try_into().unwrap_or(0);

    //         let mut board1 = self.leaderboard.get(index1).unwrap().read();
    //         let mut board2 = self.leaderboard.get(index2).unwrap().read();

    //         board1.score = board1.score - updated_points;
    //         board2.score = board2.score - updated_points;

    //         self.leaderboard.get(index1).unwrap().write(board1);
    //         self.leaderboard.get(index2).unwrap().write(board2);

    //         // lock the points for the players
    //         player1.highest_score = player1.highest_score - updated_points;
    //         player2.highest_score = player2.highest_score - updated_points;

    //         self.users.entry(address1).write(player1);
    //         self.users.entry(address2).write(player2);

    //         // update the challenge details
    //         let mut challenge_details = self.challenges.entry(challenge_id).read();
    //         challenge_details.challenger.points_locked += updated_points;
    //         challenge_details.challenged.points_locked += updated_points;

    //         self.challenges.entry(challenge_id).write(challenge_details);
    //     }
    // }
}
