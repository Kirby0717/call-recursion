use std::{cell::RefCell, pin::Pin, rc::Rc};

pub struct Output<T> {
    state: Rc<RefCell<Option<T>>>,
}
impl<T> Default for Output<T> {
    fn default() -> Self {
        Self {
            state: Rc::new(RefCell::new(None)),
        }
    }
}
impl<T: Unpin> Future for Output<T> {
    type Output = T;
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(t) = self.get_mut().state.take() {
            std::task::Poll::Ready(t)
        }
        else {
            std::task::Poll::Pending
        }
    }
}

struct FutureWrapper<F: Future> {
    future: F,
    state: Rc<RefCell<Option<F::Output>>>,
}
impl<F: Future> Future for FutureWrapper<F> {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let future = unsafe {
            Pin::new_unchecked(
                &mut Pin::get_unchecked_mut(self.as_mut()).future,
            )
        };
        future.poll(cx).map(|out| {
            *self.state.borrow_mut() = Some(out);
        })
    }
}
impl<F> FutureWrapper<F>
where
    F: Future,
    F::Output: Unpin,
{
    fn new(f: F) -> (FutureWrapper<F>, Output<F::Output>) {
        let output = Output::default();
        (
            FutureWrapper {
                future: f,
                state: output.state.clone(),
            },
            output,
        )
    }
}

thread_local! {
    static RECURSION_TEM: RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>> = const { RefCell::new(None) };
}

pub trait FutureRecursion
where
    Self: Future,
{
    fn start_recursion(self) -> Self::Output;
    fn recursion(self) -> Output<Self::Output>;
}

mod noop_waker {
    unsafe fn noop_clone(_data: *const ()) -> std::task::RawWaker {
        noop_raw_waker()
    }
    unsafe fn noop(_data: *const ()) {}
    const NOOP_WAKER_VTABLE: std::task::RawWakerVTable =
        std::task::RawWakerVTable::new(noop_clone, noop, noop, noop);
    const fn noop_raw_waker() -> std::task::RawWaker {
        std::task::RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
    }
    #[inline]
    pub fn noop_waker() -> std::task::Waker {
        unsafe { std::task::Waker::from_raw(noop_raw_waker()) }
    }
}

impl<F> FutureRecursion for F
where
    F: Future + 'static,
    F::Output: Unpin,
{
    fn start_recursion(self) -> Self::Output {
        let tem = RECURSION_TEM.replace(None);

        let waker = noop_waker::noop_waker();
        let mut context = std::task::Context::from_waker(&waker);
        let mut stack: Vec<Pin<Box<dyn Future<Output = ()>>>> = vec![];

        let (f, output) = FutureWrapper::new(self);
        stack.push(Box::pin(f));
        while let Some(l) = stack.last_mut() {
            match l.as_mut().poll(&mut context) {
                std::task::Poll::Ready(_) => {
                    stack.pop();
                }
                std::task::Poll::Pending => {
                    if let Some(f) = RECURSION_TEM.replace(None) {
                        stack.push(f);
                    }
                }
            }
        }

        RECURSION_TEM.set(tem);

        output.state.take().unwrap()
    }
    fn recursion(self) -> Output<Self::Output> {
        let (fw, output) = FutureWrapper::new(self);
        if RECURSION_TEM.replace(Some(Box::pin(fw))).is_some() {
            panic!("incorrect recursion");
        }
        output
    }
}
