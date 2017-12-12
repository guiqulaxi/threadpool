
pub trait DoTask<R> {
    fn do_task(&self) -> R;
}