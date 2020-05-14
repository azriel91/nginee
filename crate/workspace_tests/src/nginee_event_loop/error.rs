#[cfg(test)]
mod tests {
    use nginee::event_loop::Error;

    #[test]
    fn display_returns_human_readable_message() {
        assert_eq!(
            "FPS must be greater than zero.",
            Error::RateLimitFpsZero.to_string()
        )
    }
}
