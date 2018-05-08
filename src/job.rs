
pub trait FnBox {
    fn exec_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn exec_box(self: Box<F>){
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

pub enum ThreadJob {
    ///A job this thread can execute
    Do(Job),
    ///If this thread should end
    End,
}
