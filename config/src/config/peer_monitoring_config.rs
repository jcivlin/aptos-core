// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::config::{
    config_optimizer::ConfigOptimizer, config_sanitizer::ConfigSanitizer,
    node_config_loader::NodeType, utils::is_network_perf_test_enabled, Error, NodeConfig,
};
use aptos_types::chain_id::ChainId;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PeerMonitoringServiceConfig {
    pub enable_peer_monitoring_client: bool, // Whether or not to spawn the monitoring client
    pub latency_monitoring: LatencyMonitoringConfig,
    pub max_concurrent_requests: u64, // Max num of concurrent server tasks
    pub max_network_channel_size: u64, // Max num of pending network messages
    pub max_num_response_bytes: u64,  // Max num of bytes in a (serialized) response
    pub max_request_jitter_ms: u64, // Max amount of jitter (ms) that a request will be delayed for
    pub metadata_update_interval_ms: u64, // The interval (ms) between metadata updates
    pub network_monitoring: NetworkMonitoringConfig,
    pub node_monitoring: NodeMonitoringConfig,
    pub peer_monitor_interval_ms: u64, // The interval (ms) between peer monitor executions
    pub performance_monitoring: PerformanceMonitoringConfig,
}

impl Default for PeerMonitoringServiceConfig {
    fn default() -> Self {
        Self {
            enable_peer_monitoring_client: false,
            latency_monitoring: LatencyMonitoringConfig::default(),
            max_concurrent_requests: 1000,
            max_network_channel_size: 1000,
            max_num_response_bytes: 100 * 1024, // 100 KB
            max_request_jitter_ms: 1000,        // Monitoring requests are very infrequent
            metadata_update_interval_ms: 5000,
            network_monitoring: NetworkMonitoringConfig::default(),
            node_monitoring: NodeMonitoringConfig::default(),
            peer_monitor_interval_ms: 1000,
            performance_monitoring: PerformanceMonitoringConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatencyMonitoringConfig {
    pub latency_ping_interval_ms: u64, // The interval (ms) between latency pings for each peer
    pub latency_ping_timeout_ms: u64,  // The timeout (ms) for each latency ping
    pub max_latency_ping_failures: u64, // Max ping failures before the peer connection fails
    pub max_num_latency_pings_to_retain: usize, // The max latency pings to retain per peer
}

impl Default for LatencyMonitoringConfig {
    fn default() -> Self {
        Self {
            latency_ping_interval_ms: 30_000, // 30 seconds
            latency_ping_timeout_ms: 20_000,  // 20 seconds
            max_latency_ping_failures: 3,
            max_num_latency_pings_to_retain: 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkMonitoringConfig {
    pub network_info_request_interval_ms: u64, // The interval (ms) between network info requests
    pub network_info_request_timeout_ms: u64,  // The timeout (ms) for each network info request
}

impl Default for NetworkMonitoringConfig {
    fn default() -> Self {
        Self {
            network_info_request_interval_ms: 60_000, // 1 minute
            network_info_request_timeout_ms: 10_000,  // 10 seconds
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NodeMonitoringConfig {
    pub node_info_request_interval_ms: u64, // The interval (ms) between node info requests
    pub node_info_request_timeout_ms: u64,  // The timeout (ms) for each node info request
}

impl Default for NodeMonitoringConfig {
    fn default() -> Self {
        Self {
            node_info_request_interval_ms: 20_000, // 20 seconds
            node_info_request_timeout_ms: 10_000,  // 10 seconds
        }
    }
}

// Note: to enable performance monitoring, the compilation feature "network-perf-test" is required.
// Simply enabling the config values here will not enable performance monitoring.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceMonitoringConfig {
    pub enable_direct_send_testing: bool, // Whether or not to enable direct send test mode
    pub direct_send_data_size: u64,       // The amount of data to send in each request
    pub direct_send_interval_usec: u64,   // The interval (microseconds) between requests
    pub enable_rpc_testing: bool,         // Whether or not to enable RPC test mode
    pub rpc_data_size: u64,               // The amount of data to send in each RPC request
    pub rpc_interval_usec: u64,           // The interval (microseconds) between RPC requests
    pub rpc_timeout_ms: u64,              // The timeout (ms) for each RPC request
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_direct_send_testing: false, // Disabled by default
            direct_send_data_size: 512 * 1024, // 512KB
            direct_send_interval_usec: 1000,   // 1ms
            enable_rpc_testing: false,         // Disabled by default
            rpc_data_size: 512 * 1024,         // 512KB
            rpc_interval_usec: 1000,           // 1ms
            rpc_timeout_ms: 10_000,            // 10 seconds
        }
    }
}

impl ConfigSanitizer for PeerMonitoringServiceConfig {
    fn sanitize(
        node_config: &mut NodeConfig,
        _node_type: NodeType,
        chain_id: ChainId,
    ) -> Result<(), Error> {
        let sanitizer_name = Self::get_sanitizer_name();
        let peer_monitoring_config = &node_config.peer_monitoring_service;

        // Verify the peer monitoring service is not enabled in mainnet
        if chain_id.is_mainnet() && peer_monitoring_config.enable_peer_monitoring_client {
            return Err(Error::ConfigSanitizerFailed(
                sanitizer_name,
                "The peer monitoring service is not enabled in mainnet!".to_string(),
            ));
        };

        // Verify that performance monitoring is not enabled in mainnet
        let performance_monitoring_config = &peer_monitoring_config.performance_monitoring;
        if chain_id.is_mainnet()
            && (is_network_perf_test_enabled()
                || performance_monitoring_config.enable_direct_send_testing
                || performance_monitoring_config.enable_rpc_testing)
        {
            return Err(Error::ConfigSanitizerFailed(
                sanitizer_name,
                "Performance monitoring should not be enabled in mainnet!".to_string(),
            ));
        };

        Ok(())
    }
}

impl ConfigOptimizer for PeerMonitoringServiceConfig {
    fn optimize(
        node_config: &mut NodeConfig,
        local_config_yaml: &Value,
        _node_type: NodeType,
        _chain_id: ChainId,
    ) -> Result<bool, Error> {
        let peer_monitoring_config = &mut node_config.peer_monitoring_service;
        let local_performance_config_yaml =
            &local_config_yaml["peer_monitoring_service"]["performance_monitoring"];

        // Enable RPC testing if the network-perf-test feature is enabled
        let mut modified_config = false;
        if local_performance_config_yaml["enable_rpc_testing"].is_null()
            && is_network_perf_test_enabled()
        {
            peer_monitoring_config
                .performance_monitoring
                .enable_rpc_testing = true;
            modified_config = true;
        }

        Ok(modified_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_enabled_monitoring_config() {
        // Create a monitoring config with an enabled monitoring client
        let mut node_config = NodeConfig {
            peer_monitoring_service: PeerMonitoringServiceConfig {
                enable_peer_monitoring_client: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Verify the config passes sanitization for testnet
        PeerMonitoringServiceConfig::sanitize(
            &mut node_config,
            NodeType::PublicFullnode,
            ChainId::testnet(),
        )
        .unwrap();

        // Verify the config fails sanitization for mainnet
        let error = PeerMonitoringServiceConfig::sanitize(
            &mut node_config,
            NodeType::PublicFullnode,
            ChainId::mainnet(),
        )
        .unwrap_err();
        assert!(matches!(error, Error::ConfigSanitizerFailed(_, _)));
    }
}
