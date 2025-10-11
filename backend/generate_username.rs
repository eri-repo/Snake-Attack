use rand::prelude::IndexedRandom;
use rand::Rng;
// Arrays of words for username generation
const ADJECTIVES: &[&str] = &[
    "Ancient", "Arcane", "Astral", "Brave", "Brilliant", "Celestial", "Clever", "Cosmic",
    "Crimson", "Crystal", "Dark", "Divine", "Ebon", "Eternal", "Fierce", "Fiery",
    "Frozen", "Gentle", "Golden", "Hidden", "Infernal", "Iron", "Jade", "Light",
    "Lunar", "Majestic", "Mighty", "Mystic", "Noble", "Obsidian", "Phantom", "Quick",
    "Radiant", "Sapphire", "Shadow", "Sharp", "Silver", "Solar", "Storm", "Strong",
    "Swift", "Thunder", "Titan", "Valiant", "Void", "Whispering", "Wild", "Wise",
    "Wrathful", "Zealous"
];

const NOUNS: &[&str] = &[
    "Archer", "Assassin", "Avenger", "Barbarian", "Bear", "Blade", "Cobra", "Crown",
    "Defender", "Dolphin", "Dragon", "Eagle", "Emperor", "Falcon", "Fox", "Guardian",
    "Hawk", "Hunter", "Hydra", "Knight", "Legend", "Lion", "Mage", "Monarch",
    "Moon", "Owl", "Panther", "Phoenix", "Ranger", "Raven", "Sage", "Scorpion",
    "Scout", "Serpent", "Shadow", "Shark", "Shield", "Sorcerer", "Spear", "Specter",
    "Spider", "Spirit", "Star", "Storm", "Sun", "Sword", "Tiger", "Titan",
    "Valkyrie", "Viper", "Warrior", "Whale", "Wolf", "Wraith"
];

/// Generates a random username by combining an adjective, noun, and random number
/// Returns a String in the format `AdjectiveNounNumber` (e.g., `SwiftMage4721`)
pub fn generate_random_username() -> String {
    let mut rng = rand::rng();
    let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"brave");
    let noun = NOUNS.choose(&mut rng).unwrap_or(&"warrior");
    let number = rng.random_range(1..=9999);
    format!("{}_{}{}", adjective, noun, number)
}