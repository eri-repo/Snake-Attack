export const contractABI = [
  {
    type: "impl",
    name: "Starknake",
    interface_name: "starknake::interfaces::IStarknake",
  },
  {
    type: "struct",
    name: "core::integer::u256",
    members: [
      { name: "low", type: "core::integer::u128" },
      { name: "high", type: "core::integer::u128" },
    ],
  },
  {
    type: "interface",
    name: "starknake::interfaces::IStarknake",
    items: [
      {
        type: "function",
        name: "player_registers",
        inputs: [
          {
            name: "player_address",
            type: "core::starknet::contract_address::ContractAddress",
          },
          { name: "username", type: "core::felt252" },
        ],
        outputs: [],
        state_mutability: "external",
      },
      {
        type: "function",
        name: "player_update_username",
        inputs: [{ name: "username", type: "core::felt252" }],
        outputs: [],
        state_mutability: "external",
      },
      {
        type: "function",
        name: "update_player_score",
        inputs: [
          {
            name: "address",
            type: "core::starknet::contract_address::ContractAddress",
          },
          { name: "current_score", type: "core::integer::u256" },
        ],
        outputs: [],
        state_mutability: "external",
      },
    ],
  },
  {
    type: "constructor",
    name: "constructor",
    inputs: [
      {
        name: "owner",
        type: "core::starknet::contract_address::ContractAddress",
      },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::ContractCreated",
    kind: "struct",
    members: [
      {
        name: "contract_address",
        type: "core::starknet::contract_address::ContractAddress",
        kind: "key",
      },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::PlayerRegistered",
    kind: "struct",
    members: [
      {
        name: "player_address",
        type: "core::starknet::contract_address::ContractAddress",
        kind: "key",
      },
      { name: "username", type: "core::felt252", kind: "key" },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::UsernameUpdted",
    kind: "struct",
    members: [
      { name: "old_username", type: "core::felt252", kind: "key" },
      { name: "new_username", type: "core::felt252", kind: "data" },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::ChallengeCreated",
    kind: "struct",
    members: [
      { name: "challenge_id", type: "core::felt252", kind: "key" },
      {
        name: "challenged",
        type: "core::starknet::contract_address::ContractAddress",
        kind: "key",
      },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::ScoreUpdated",
    kind: "struct",
    members: [{ name: "score", type: "core::integer::u256", kind: "key" }],
  },
  {
    type: "event",
    name: "starknake::events::Events::CounterUpdateChallenge",
    kind: "struct",
    members: [{ name: "challenge_id", type: "core::felt252", kind: "key" }],
  },
  {
    type: "event",
    name: "starknake::events::Events::ChallengeUpdated",
    kind: "struct",
    members: [
      { name: "challenge_id", type: "core::felt252", kind: "key" },
      {
        name: "winner",
        type: "core::starknet::contract_address::ContractAddress",
        kind: "key",
      },
    ],
  },
  {
    type: "event",
    name: "starknake::events::Events::ChallengeAccepted",
    kind: "struct",
    members: [{ name: "challenge_id", type: "core::felt252", kind: "key" }],
  },
  {
    type: "event",
    name: "starknake::starknake::Starknake::Event",
    kind: "enum",
    variants: [
      {
        name: "ContractCreated",
        type: "starknake::events::Events::ContractCreated",
        kind: "nested",
      },
      {
        name: "PlayerRegistered",
        type: "starknake::events::Events::PlayerRegistered",
        kind: "nested",
      },
      {
        name: "UsernameUpdted",
        type: "starknake::events::Events::UsernameUpdted",
        kind: "nested",
      },
      {
        name: "ChallengeCreated",
        type: "starknake::events::Events::ChallengeCreated",
        kind: "nested",
      },
      {
        name: "ScoreUpdated",
        type: "starknake::events::Events::ScoreUpdated",
        kind: "nested",
      },
      {
        name: "CounterUpdateChallenge",
        type: "starknake::events::Events::CounterUpdateChallenge",
        kind: "nested",
      },
      {
        name: "ChallengeUpdated",
        type: "starknake::events::Events::ChallengeUpdated",
        kind: "nested",
      },
      {
        name: "ChallengeAccepted",
        type: "starknake::events::Events::ChallengeAccepted",
        kind: "nested",
      },
    ],
  },
];
