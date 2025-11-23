//! Flamegraph format exporter
//!
//! Exports async-inspect data to flamegraph folded stack format for visualization
//! with tools like inferno, speedscope, or Brendan Gregg's flamegraph.pl
//!
//! Format: `stack1;stack2;stack3 count`
//! Each line represents a call stack with semicolon-separated frames and a sample count

use crate::inspector::Inspector;
use crate::timeline::EventKind;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Flamegraph format exporter
pub struct FlamegraphExporter;

impl FlamegraphExporter {
    /// Export to flamegraph folded stack format file
    pub fn export_to_file<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        let folded = Self::generate_folded_stacks(inspector);
        let mut file = File::create(path)?;
        file.write_all(folded.as_bytes())?;
        Ok(())
    }

    /// Export to flamegraph folded stack format string
    #[must_use] 
    pub fn export_to_string(inspector: &Inspector) -> String {
        Self::generate_folded_stacks(inspector)
    }

    /// Generate folded stack format from inspector data
    fn generate_folded_stacks(inspector: &Inspector) -> String {
        let mut output = String::new();
        let events = inspector.get_events();

        // Track call stacks and durations
        let mut task_stacks: HashMap<u64, Vec<String>> = HashMap::new();
        let mut stack_samples: HashMap<String, u64> = HashMap::new();

        // Build stacks from events
        for event in events {
            let task_id = event.task_id.as_u64();

            match &event.kind {
                EventKind::TaskSpawned { name, parent, .. } => {
                    // Initialize stack for this task
                    let mut stack = Vec::new();

                    // Add parent context if available
                    if let Some(parent_id) = parent {
                        if let Some(parent_stack) = task_stacks.get(&parent_id.as_u64()) {
                            stack.extend(parent_stack.clone());
                        }
                    }

                    // Add this task to the stack
                    stack.push(Self::sanitize_frame_name(name));
                    task_stacks.insert(task_id, stack);
                }

                EventKind::AwaitEnded {
                    await_point,
                    duration,
                } => {
                    if let Some(stack) = task_stacks.get_mut(&task_id) {
                        // Add await point to stack
                        stack.push(Self::sanitize_frame_name(await_point));

                        // Record sample with duration
                        let stack_str = stack.join(";");
                        let duration_ms = duration.as_millis() as u64;

                        *stack_samples.entry(stack_str).or_insert(0) += duration_ms;

                        // Pop await point from stack
                        stack.pop();
                    }
                }

                EventKind::PollEnded { duration } => {
                    if let Some(stack) = task_stacks.get(&task_id) {
                        // Record poll time
                        let mut poll_stack = stack.clone();
                        poll_stack.push("poll".to_string());

                        let stack_str = poll_stack.join(";");
                        let duration_ms = duration.as_millis() as u64;

                        *stack_samples.entry(stack_str).or_insert(0) += duration_ms;
                    }
                }

                EventKind::TaskCompleted { duration } => {
                    if let Some(stack) = task_stacks.get(&task_id) {
                        // Record total task time
                        let stack_str = stack.join(";");
                        let duration_ms = duration.as_millis() as u64;

                        *stack_samples.entry(stack_str).or_insert(0) += duration_ms;
                    }
                }

                _ => {}
            }
        }

        // Sort stacks by sample count (descending) for better visualization
        let mut sorted_stacks: Vec<_> = stack_samples.into_iter().collect();
        sorted_stacks.sort_by(|a, b| b.1.cmp(&a.1));

        // Generate folded output
        for (stack, count) in sorted_stacks {
            if count > 0 {
                output.push_str(&format!("{stack} {count}\n"));
            }
        }

        output
    }

    /// Sanitize frame names for flamegraph format
    /// Removes characters that could break the format
    fn sanitize_frame_name(name: &str) -> String {
        name.replace(';', ":")
            .replace('\n', " ")
            .replace('\r', "")
            .trim()
            .to_string()
    }

    /// Generate SVG flamegraph using inferno library (if available)
    #[cfg(feature = "flamegraph")]
    pub fn generate_svg<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        use inferno::flamegraph::{self, Options};

        let folded = Self::generate_folded_stacks(inspector);
        let folded_bytes = folded.as_bytes();

        let mut options = Options::default();
        options.title = "async-inspect Flamegraph".to_string();
        options.subtitle = Some("Async task execution time".to_string());

        let mut svg_output = File::create(path)?;

        flamegraph::from_reader(
            &mut options,
            folded_bytes,
            &mut svg_output,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}

/// Builder for flamegraph export with customization options
pub struct FlamegraphBuilder {
    /// Whether to include poll events
    include_polls: bool,
    /// Whether to include await points
    include_awaits: bool,
    /// Minimum duration threshold in milliseconds
    min_duration_ms: u64,
}

impl Default for FlamegraphBuilder {
    fn default() -> Self {
        Self {
            include_polls: true,
            include_awaits: true,
            min_duration_ms: 0,
        }
    }
}

impl FlamegraphBuilder {
    /// Create a new flamegraph builder
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include poll events
    #[must_use] 
    pub fn include_polls(mut self, include: bool) -> Self {
        self.include_polls = include;
        self
    }

    /// Set whether to include await points
    #[must_use] 
    pub fn include_awaits(mut self, include: bool) -> Self {
        self.include_awaits = include;
        self
    }

    /// Set minimum duration threshold (in milliseconds)
    #[must_use] 
    pub fn min_duration_ms(mut self, ms: u64) -> Self {
        self.min_duration_ms = ms;
        self
    }

    /// Build and export flamegraph
    pub fn export_to_file<P: AsRef<Path>>(
        self,
        inspector: &Inspector,
        path: P,
    ) -> io::Result<()> {
        // For now, use default implementation
        // TODO: Apply builder options when generating stacks
        FlamegraphExporter::export_to_file(inspector, path)
    }

    /// Build and export flamegraph as string
    #[must_use] 
    pub fn export_to_string(self, inspector: &Inspector) -> String {
        // For now, use default implementation
        // TODO: Apply builder options when generating stacks
        FlamegraphExporter::export_to_string(inspector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_frame_name() {
        assert_eq!(
            FlamegraphExporter::sanitize_frame_name("simple"),
            "simple"
        );
        assert_eq!(
            FlamegraphExporter::sanitize_frame_name("with;semicolon"),
            "with:semicolon"
        );
        assert_eq!(
            FlamegraphExporter::sanitize_frame_name("with\nnewline"),
            "with newline"
        );
        assert_eq!(
            FlamegraphExporter::sanitize_frame_name("  trimmed  "),
            "trimmed"
        );
    }

    #[test]
    fn test_flamegraph_builder() {
        let builder = FlamegraphBuilder::new()
            .include_polls(false)
            .include_awaits(true)
            .min_duration_ms(10);

        assert!(!builder.include_polls);
        assert!(builder.include_awaits);
        assert_eq!(builder.min_duration_ms, 10);
    }
}
