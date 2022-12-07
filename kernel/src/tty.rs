pub fn run() {
    util::ei();
    loop {
        util::halt();
        info!("interrupt!");
    }
}
