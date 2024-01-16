// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            5
// Async Callback (empty):               1
// Total number of exported functions:   7

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    composable_tasks
    (
        init => init
        upgrade => upgrade
        composeTasks => compose_tasks
        setWrapEgldAddr => set_wrap_egld_address
        setRouterAddr => set_router_address
        getPair => get_pair
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
