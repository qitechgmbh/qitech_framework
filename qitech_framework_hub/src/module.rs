pub trait DatabaseModule {
    fn init() -> Self;
    async fn start();
}

pub trait ApiModule {
    fn init() -> Self;
    async fn start();
}