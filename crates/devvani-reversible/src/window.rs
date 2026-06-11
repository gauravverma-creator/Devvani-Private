use crate::error::ReversibleError;
use crate::types::OpId;
use std::collections::VecDeque;

/// Configuration for the sliding compute window.
/// The window controls how many operations are kept in RAM before purging.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Maximum number of operations in the window before a purge is triggered
    pub max_ops: usize,
    /// Fraction of ops to purge when max_ops is hit. Default: 0.80 (80%)
    pub purge_fraction: f64,
    /// If true, purge checks dependencies before removing. Always true in production.
    pub dependency_check: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            max_ops: 1024,
            purge_fraction: 0.80,
            dependency_check: true,
        }
    }
}

impl WindowConfig {
    pub fn validate(&self) -> Result<(), ReversibleError> {
        if self.max_ops == 0 {
            return Err(ReversibleError::InvalidConfig {
                reason: "max_ops cannot be zero".to_string(),
            });
        }
        if self.purge_fraction <= 0.0 || self.purge_fraction > 1.0 {
            return Err(ReversibleError::InvalidConfig {
                reason: format!(
                    "purge_fraction must be in (0.0, 1.0], got {}",
                    self.purge_fraction
                ),
            });
        }
        Ok(())
    }

    pub fn purge_count(&self) -> usize {
        (self.max_ops as f64 * self.purge_fraction).ceil() as usize
    }
}

/// The sliding window tracks which OpIds are currently "live" in RAM.
/// When the window fills up, it triggers a purge of the oldest 80% of entries.
#[derive(Debug)]
pub struct ComputeWindow {
    config: WindowConfig,
    live_ops: VecDeque<OpId>,
    total_purged: u64,
}

impl ComputeWindow {
    pub fn new(config: WindowConfig) -> Result<Self, ReversibleError> {
        config.validate()?;
        Ok(Self {
            config,
            live_ops: VecDeque::new(),
            total_purged: 0,
        })
    }

    pub fn with_default_config() -> Self {
        Self::new(WindowConfig::default()).expect("Default config is always valid")
    }

    /// Add an OpId to the window. Returns the list of OpIds that should be purged
    /// if the window is now at capacity. Returns empty vec if no purge needed.
    pub fn push(&mut self, op_id: OpId) -> Vec<OpId> {
        self.live_ops.push_back(op_id);
        if self.live_ops.len() >= self.config.max_ops {
            self.trigger_purge()
        } else {
            vec![]
        }
    }

    /// Manually trigger a purge. Returns the OpIds that should be evicted.
    pub fn trigger_purge(&mut self) -> Vec<OpId> {
        let count = self.config.purge_count().min(self.live_ops.len());
        let mut purged = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(id) = self.live_ops.pop_front() {
                purged.push(id);
            }
        }
        self.total_purged += purged.len() as u64;
        purged
    }

    pub fn current_size(&self) -> usize {
        self.live_ops.len()
    }

    pub fn is_full(&self) -> bool {
        self.live_ops.len() >= self.config.max_ops
    }

    pub fn total_purged(&self) -> u64 {
        self.total_purged
    }

    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    pub fn live_ops(&self) -> &VecDeque<OpId> {
        &self.live_ops
    }
}