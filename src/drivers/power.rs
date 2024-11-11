pub fn init(resources: crate::drivers::resources::PowerDriverResources) {
    // Enable the DC/DC converter
    resources.power.dcdcen.write(|w| w.dcdcen().enabled());
}
