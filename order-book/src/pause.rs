// stolen from the framework, but that's with #[only_owner] instead of #[only_admin]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PauseModule {
    #[inline]
    fn is_paused(&self) -> bool {
        self.paused_status().get()
    }

    #[inline]
    fn not_paused(&self) -> bool {
        !self.is_paused()
    }

    #[inline]
    fn set_paused(&self, paused: bool) {
        self.paused_status().set(paused);
    }

    fn require_paused(&self) {
        require!(self.is_paused(), "Contract is not paused");
    }

    fn require_not_paused(&self) {
        require!(self.not_paused(), "Contract is paused");
    }

    #[event("pauseContract")]
    fn pause_event(&self);

    #[event("unpauseContract")]
    fn unpause_event(&self);

    #[view(isPaused)]
    #[storage_mapper("pause_module:paused")]
    fn paused_status(&self) -> SingleValueMapper<bool>;
}
