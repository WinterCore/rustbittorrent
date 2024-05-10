pub trait Extension {
    const NAME: &'static str;

    async fn process_packet(&self, data: &[u8]) -> ();
}
