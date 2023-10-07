pub trait Strategy {
    fn new();

    fn on_data();

    fn on_order();
}
