multiversx_sc::imports!();

pub struct PairConfig<M: ManagedTypeApi> {
    pub lp_token_id: TokenIdentifier<M>,
    pub first_token_id: TokenIdentifier<M>,
    pub second_token_id: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait PairsConfigModule:
    read_external_storage::ReadExternalStorageModule + utils::UtilsModule
{
    fn get_pair_config(&self, pair_address: &ManagedAddress) -> PairConfig<Self::Api> {
        let lp_token_id = self.get_lp_token_id_mapper(pair_address.clone()).get();
        let first_token_id = self.get_first_token_id_mapper(pair_address.clone()).get();
        let second_token_id = self.get_second_token_id_mapper(pair_address.clone()).get();

        self.require_valid_token_id(&lp_token_id);
        self.require_valid_token_id(&first_token_id);
        self.require_valid_token_id(&second_token_id);

        PairConfig {
            lp_token_id,
            first_token_id,
            second_token_id,
        }
    }
}
