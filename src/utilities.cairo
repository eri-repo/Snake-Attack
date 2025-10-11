pub mod Utilities {
    use starknet::ContractAddress;
    use crate::errors::Errors::*;

    pub fn address_zero_check(address: ContractAddress) {
        assert(address != 0x0.try_into().unwrap(), ZERO_ADDRESS);
    }
}
