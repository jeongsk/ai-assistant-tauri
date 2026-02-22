//! Resource Monitor for Plugin Execution
//!
//! This module provides real-time resource tracking for plugin instances.

use crate::plugins::ResourceLimits;
use serde::{Deserialize, Serialize};

/// Resource metrics collected during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Plugin instance ID
    pub instance_id: String,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Number of syscalls made
    pub syscall_count: u64,
    /// Number of file operations
    pub file_ops_count: u64,
    /// Number of network operations
    pub network_ops_count: u64,
    /// Timestamp of measurement
    pub timestamp: u64,
    /// Fuel consumed (for WASM instances)
    #[cfg(feature = "wasm")]
    pub fuel_consumed: u64,
}

impl ResourceMetrics {
    /// Create new metrics
    pub fn new(instance_id: String) -> Self {
        Self {
            instance_id,
            memory_bytes: 0,
            cpu_time_ms: 0,
            syscall_count: 0,
            file_ops_count: 0,
            network_ops_count: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            #[cfg(feature = "wasm")]
            fuel_consumed: 0,
        }
    }

    /// Update metrics from WASM execution
    #[cfg(feature = "wasm")]
    pub fn update_from_wasm(&mut self, fuel_consumed: u64, memory_bytes: u64) {
        self.fuel_consumed = fuel_consumed;
        self.memory_bytes = memory_bytes;
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if metrics exceed limits
    pub fn exceeds_limits(&self, limits: &ResourceLimits) -> bool {
        // Check memory limit (convert MB to bytes)
        if self.memory_bytes > (limits.max_memory_mb as u64 * 1024 * 1024) {
            return true;
        }

        // Check execution time limit
        if self.cpu_time_ms > limits.max_execution_time_ms as u64 {
            return true;
        }

        false
    }
}

/// Resource monitor for tracking plugin resource usage
pub struct ResourceMonitor {
    /// Current metrics for each instance
    metrics: Vec<ResourceMetrics>,
    /// Historical metrics (last 100 measurements per instance)
    history: Vec<ResourceMetrics>,
    /// Maximum history size
    max_history: usize,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Start monitoring an instance
    pub fn start_monitoring(&mut self, instance_id: String) {
        let metrics = ResourceMetrics::new(instance_id);
        self.metrics.push(metrics);
    }

    /// Stop monitoring an instance
    pub fn stop_monitoring(&mut self, instance_id: &str) -> Option<ResourceMetrics> {
        let pos = self.metrics.iter().position(|m| m.instance_id == instance_id)?;
        Some(self.metrics.remove(pos))
    }

    /// Update metrics for an instance
    pub fn update_metrics(&mut self, instance_id: &str, update: MetricUpdate) {
        if let Some(metrics) = self.metrics.iter_mut().find(|m| m.instance_id == instance_id) {
            match update {
                MetricUpdate::Memory(bytes) => metrics.memory_bytes = bytes,
                MetricUpdate::CpuTime(ms) => metrics.cpu_time_ms = ms,
                MetricUpdate::Syscall => metrics.syscall_count += 1,
                MetricUpdate::FileOp => metrics.file_ops_count += 1,
                MetricUpdate::NetworkOp => metrics.network_ops_count += 1,
                #[cfg(feature = "wasm")]
                MetricUpdate::Fuel(fuel) => metrics.fuel_consumed = fuel,
            }

            // Update timestamp
            metrics.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Save to history
            let snapshot = metrics.clone();
            self.history.push(snapshot);

            // Prune history
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Update metrics from WASM execution
    #[cfg(feature = "wasm")]
    pub fn update_from_wasm(&mut self, instance_id: &str, fuel_consumed: u64, memory_bytes: u64) {
        if let Some(metrics) = self.metrics.iter_mut().find(|m| m.instance_id == instance_id) {
            metrics.update_from_wasm(fuel_consumed, memory_bytes);

            // Save to history
            let snapshot = metrics.clone();
            self.history.push(snapshot);

            // Prune history
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Get current metrics for an instance
    pub fn get_metrics(&self, instance_id: &str) -> Option<&ResourceMetrics> {
        self.metrics.iter().find(|m| m.instance_id == instance_id)
    }

    /// Get all current metrics
    pub fn get_all_metrics(&self) -> &[ResourceMetrics] {
        &self.metrics
    }

    /// Get historical metrics for an instance
    pub fn get_history(&self, instance_id: &str) -> Vec<&ResourceMetrics> {
        self.history
            .iter()
            .filter(|m| m.instance_id == instance_id)
            .collect()
    }

    /// Get summary statistics for an instance
    pub fn get_summary(&self, instance_id: &str) -> Option<ResourceSummary> {
        let current = self.get_metrics(instance_id)?;
        let history = self.get_history(instance_id);

        if history.is_empty() {
            return Some(ResourceSummary {
                instance_id: instance_id.to_string(),
                current_memory: current.memory_bytes,
                peak_memory: current.memory_bytes,
                current_cpu_time: current.cpu_time_ms,
                total_syscalls: current.syscall_count,
                total_file_ops: current.file_ops_count,
                total_network_ops: current.network_ops_count,
                measurement_count: 1,
                #[cfg(feature = "wasm")]
                total_fuel_consumed: current.fuel_consumed,
            });
        }

        let peak_memory = history.iter().map(|m| m.memory_bytes).max().unwrap_or(0);
        let total_syscalls = history.iter().map(|m| m.syscall_count).sum();

        #[cfg(feature = "wasm")]
        let total_fuel = history.iter().map(|m| m.fuel_consumed).sum();

        #[cfg(not(feature = "wasm"))]
        let _total_fuel = 0;

        Some(ResourceSummary {
            instance_id: instance_id.to_string(),
            current_memory: current.memory_bytes,
            peak_memory,
            current_cpu_time: current.cpu_time_ms,
            total_syscalls,
            total_file_ops: history.iter().map(|m| m.file_ops_count).sum(),
            total_network_ops: history.iter().map(|m| m.network_ops_count).sum(),
            measurement_count: history.len(),
            #[cfg(feature = "wasm")]
            total_fuel_consumed: total_fuel,
        })
    }

    /// Check all monitored instances against limits
    pub fn check_limits(&self, limits: &ResourceLimits) -> Vec<String> {
        self.metrics
            .iter()
            .filter(|m| m.exceeds_limits(limits))
            .map(|m| m.instance_id.clone())
            .collect()
    }

    /// Get total resource usage across all instances
    pub fn get_total_usage(&self) -> TotalResourceUsage {
        let total_memory: u64 = self.metrics.iter().map(|m| m.memory_bytes).sum();
        let total_cpu: u64 = self.metrics.iter().map(|m| m.cpu_time_ms).sum();
        let total_syscalls: u64 = self.metrics.iter().map(|m| m.syscall_count).sum();

        #[cfg(feature = "wasm")]
        let total_fuel: u64 = self.metrics.iter().map(|m| m.fuel_consumed).sum();

        #[cfg(not(feature = "wasm"))]
        let _total_fuel = 0;

        TotalResourceUsage {
            instance_count: self.metrics.len(),
            total_memory,
            total_cpu,
            total_syscalls,
            #[cfg(feature = "wasm")]
            total_fuel_consumed: total_fuel,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric update type
pub enum MetricUpdate {
    Memory(u64),
    CpuTime(u64),
    Syscall,
    FileOp,
    NetworkOp,
    #[cfg(feature = "wasm")]
    Fuel(u64),
}

/// Resource summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    pub instance_id: String,
    pub current_memory: u64,
    pub peak_memory: u64,
    pub current_cpu_time: u64,
    pub total_syscalls: u64,
    pub total_file_ops: u64,
    pub total_network_ops: u64,
    pub measurement_count: usize,
    #[cfg(feature = "wasm")]
    pub total_fuel_consumed: u64,
}

/// Total resource usage across all instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalResourceUsage {
    pub instance_count: usize,
    pub total_memory: u64,
    pub total_cpu: u64,
    pub total_syscalls: u64,
    #[cfg(feature = "wasm")]
    pub total_fuel_consumed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ResourceMonitor::new();
        assert_eq!(monitor.metrics.len(), 0);
        assert_eq!(monitor.history.len(), 0);
    }

    #[test]
    fn test_start_monitoring() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("test-instance".to_string());
        assert_eq!(monitor.metrics.len(), 1);
    }

    #[test]
    fn test_update_metrics() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("test-instance".to_string());

        monitor.update_metrics("test-instance", MetricUpdate::Memory(1024));
        monitor.update_metrics("test-instance", MetricUpdate::Syscall);

        let metrics = monitor.get_metrics("test-instance").unwrap();
        assert_eq!(metrics.memory_bytes, 1024);
        assert_eq!(metrics.syscall_count, 1);
    }

    #[test]
    fn test_exceeds_limits() {
        let limits = ResourceLimits {
            max_memory_mb: 1,
            max_cpu_percent: 50,
            max_execution_time_ms: 1000,
            max_file_size_mb: 10,
        };

        let mut metrics = ResourceMetrics::new("test".to_string());
        assert!(!metrics.exceeds_limits(&limits));

        metrics.memory_bytes = 2 * 1024 * 1024; // 2MB
        assert!(metrics.exceeds_limits(&limits));
    }

    #[test]
    fn test_stop_monitoring() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("test-instance".to_string());
        assert_eq!(monitor.metrics.len(), 1);

        let metrics = monitor.stop_monitoring("test-instance").unwrap();
        assert_eq!(metrics.instance_id, "test-instance");
        assert_eq!(monitor.metrics.len(), 0);
    }

    #[test]
    fn test_get_summary() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("test-instance".to_string());

        monitor.update_metrics("test-instance", MetricUpdate::Memory(1024));
        monitor.update_metrics("test-instance", MetricUpdate::Syscall);

        let summary = monitor.get_summary("test-instance").unwrap();
        assert_eq!(summary.instance_id, "test-instance");
        assert_eq!(summary.current_memory, 1024);
        assert_eq!(summary.total_syscalls, 1);
    }

    #[test]
    fn test_check_limits() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("ok-instance".to_string());
        monitor.start_monitoring("over-limit".to_string());

        // Set one instance over limit
        monitor.update_metrics("over-limit", MetricUpdate::Memory(2 * 1024 * 1024));

        let limits = ResourceLimits {
            max_memory_mb: 1,
            max_cpu_percent: 50,
            max_execution_time_ms: 1000,
            max_file_size_mb: 10,
        };

        let exceeded = monitor.check_limits(&limits);
        assert_eq!(exceeded.len(), 1);
        assert_eq!(exceeded[0], "over-limit");
    }

    #[test]
    fn test_total_usage() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("instance1".to_string());
        monitor.start_monitoring("instance2".to_string());

        monitor.update_metrics("instance1", MetricUpdate::Memory(1024));
        monitor.update_metrics("instance2", MetricUpdate::Memory(2048));

        let usage = monitor.get_total_usage();
        assert_eq!(usage.instance_count, 2);
        assert_eq!(usage.total_memory, 3072);
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_wasm_fuel_tracking() {
        let mut monitor = ResourceMonitor::new();
        monitor.start_monitoring("test-instance".to_string());

        monitor.update_from_wasm("test-instance", 5000, 1024);

        let metrics = monitor.get_metrics("test-instance").unwrap();
        assert_eq!(metrics.fuel_consumed, 5000);
        assert_eq!(metrics.memory_bytes, 1024);
    }
}
