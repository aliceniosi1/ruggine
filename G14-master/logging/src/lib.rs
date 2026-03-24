pub mod cpu;


#[cfg(test)]
mod tests {
    use super::cpu::log_cpu_usage;

    #[tokio::test]
    async fn test_log_cpu_usage() {
        // Call the logging function and ensure it returns Ok(())
        let result = log_cpu_usage().await;
        assert!(result.is_ok());
    }
}
