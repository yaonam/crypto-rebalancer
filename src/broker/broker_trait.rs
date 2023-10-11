pub trait Broker {
    fn new() -> Self;

    fn connect();

    // Methods for strat
    fn get_total_value() -> f64;

    fn place_order();

    fn cancel_order();
}