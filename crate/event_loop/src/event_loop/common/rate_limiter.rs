use governor::{
    clock::DefaultClock,
    state::{direct::NotKeyed, InMemoryState},
};

pub(crate) type RateLimiter = governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>;
