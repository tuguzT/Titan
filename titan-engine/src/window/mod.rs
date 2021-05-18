pub trait Callback<T>
    where T: Callback<T>
{
    fn new() -> T;

    fn on_create(&self);

    fn on_destroy(&self);
}
