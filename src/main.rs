#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use embassy_executor::Spawner;
use hamboo::bsp::*;

#[main]
async fn main(spawner: Spawner) {
    let _bsp = HambooBsp::new();

    // spawner.spawn(bsp::wifi_start()).ok();
    spawner.spawn(hamboo::ui::run()).ok();
}
