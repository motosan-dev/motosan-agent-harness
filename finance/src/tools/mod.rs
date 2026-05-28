pub mod get_position;
pub mod get_quote;
pub mod place_order;

pub use get_position::GetPositionTool;
pub use get_quote::GetQuoteTool;
pub use place_order::PlaceOrderTool;

pub(crate) const MOCK_QUOTES: &[(&str, f64)] = &[
    ("AAPL", 185.00),
    ("MSFT", 420.00),
    ("GOOGL", 175.00),
    ("TSLA", 245.00),
    ("NVDA", 880.00),
];

pub(crate) const MOCK_POSITIONS: &[(&str, u32, f64)] = &[
    ("AAPL", 50, 150.00),
    ("MSFT", 10, 380.00),
    ("GOOGL", 0, 0.00),
    ("TSLA", 0, 0.00),
    ("NVDA", 0, 0.00),
];

pub(crate) fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

pub(crate) fn quote_for(symbol: &str) -> Option<f64> {
    let symbol = normalize_symbol(symbol);
    MOCK_QUOTES
        .iter()
        .find_map(|(s, price)| (*s == symbol).then_some(*price))
}

pub(crate) fn position_for(symbol: &str) -> (u32, f64) {
    let symbol = normalize_symbol(symbol);
    MOCK_POSITIONS
        .iter()
        .find_map(|(s, quantity, cost_basis)| (*s == symbol).then_some((*quantity, *cost_basis)))
        .unwrap_or((0, 0.0))
}

pub(crate) fn required_string(args: &serde_json::Value, field: &str) -> Result<String, String> {
    args.get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("bad args: missing or invalid string field '{field}'"))
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut future = Box::pin(future);
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test future unexpectedly pending"),
        }
    }

    fn noop_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            raw_waker()
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}

        fn raw_waker() -> RawWaker {
            RawWaker::new(
                std::ptr::null(),
                &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
            )
        }

        unsafe { Waker::from_raw(raw_waker()) }
    }
}
