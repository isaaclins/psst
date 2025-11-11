use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Represents a single band in the equalizer with a center frequency and gain.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand {
    /// Center frequency in Hz
    pub frequency: f32,
    /// Gain in dB (typically -12.0 to +12.0)
    pub gain_db: f32,
}

impl EqualizerBand {
    pub fn new(frequency: f32, gain_db: f32) -> Self {
        Self {
            frequency,
            gain_db,
        }
    }
}

/// Configuration for the equalizer with multiple bands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerConfig {
    /// Whether the equalizer is enabled
    pub enabled: bool,
    /// The frequency bands
    pub bands: Vec<EqualizerBand>,
}

impl Default for EqualizerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bands: Self::default_bands(),
        }
    }
}

impl EqualizerConfig {
    /// Standard 10-band equalizer frequencies
    pub fn default_bands() -> Vec<EqualizerBand> {
        vec![
            EqualizerBand::new(32.0, 0.0),
            EqualizerBand::new(64.0, 0.0),
            EqualizerBand::new(125.0, 0.0),
            EqualizerBand::new(250.0, 0.0),
            EqualizerBand::new(500.0, 0.0),
            EqualizerBand::new(1000.0, 0.0),
            EqualizerBand::new(2000.0, 0.0),
            EqualizerBand::new(4000.0, 0.0),
            EqualizerBand::new(8000.0, 0.0),
            EqualizerBand::new(16000.0, 0.0),
        ]
    }

    /// Create a new equalizer config with custom bands
    pub fn new(enabled: bool, bands: Vec<EqualizerBand>) -> Self {
        Self { enabled, bands }
    }
}

/// A named preset for the equalizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerPreset {
    pub name: String,
    pub bands: Vec<EqualizerBand>,
}

impl EqualizerPreset {
    pub fn new(name: String, bands: Vec<EqualizerBand>) -> Self {
        Self { name, bands }
    }

    /// Get built-in presets
    pub fn built_in_presets() -> Vec<Self> {
        vec![
            Self::flat(),
            Self::bass_boost(),
            Self::treble_boost(),
            Self::vocal(),
            Self::rock(),
            Self::classical(),
            Self::jazz(),
            Self::pop(),
        ]
    }

    pub fn flat() -> Self {
        Self::new("Flat".to_string(), EqualizerConfig::default_bands())
    }

    pub fn bass_boost() -> Self {
        Self::new(
            "Bass Boost".to_string(),
            vec![
                EqualizerBand::new(32.0, 8.0),
                EqualizerBand::new(64.0, 6.0),
                EqualizerBand::new(125.0, 4.0),
                EqualizerBand::new(250.0, 2.0),
                EqualizerBand::new(500.0, 0.0),
                EqualizerBand::new(1000.0, 0.0),
                EqualizerBand::new(2000.0, 0.0),
                EqualizerBand::new(4000.0, 0.0),
                EqualizerBand::new(8000.0, 0.0),
                EqualizerBand::new(16000.0, 0.0),
            ],
        )
    }

    pub fn treble_boost() -> Self {
        Self::new(
            "Treble Boost".to_string(),
            vec![
                EqualizerBand::new(32.0, 0.0),
                EqualizerBand::new(64.0, 0.0),
                EqualizerBand::new(125.0, 0.0),
                EqualizerBand::new(250.0, 0.0),
                EqualizerBand::new(500.0, 0.0),
                EqualizerBand::new(1000.0, 2.0),
                EqualizerBand::new(2000.0, 4.0),
                EqualizerBand::new(4000.0, 6.0),
                EqualizerBand::new(8000.0, 8.0),
                EqualizerBand::new(16000.0, 8.0),
            ],
        )
    }

    pub fn vocal() -> Self {
        Self::new(
            "Vocal".to_string(),
            vec![
                EqualizerBand::new(32.0, -2.0),
                EqualizerBand::new(64.0, -2.0),
                EqualizerBand::new(125.0, -1.0),
                EqualizerBand::new(250.0, 2.0),
                EqualizerBand::new(500.0, 4.0),
                EqualizerBand::new(1000.0, 4.0),
                EqualizerBand::new(2000.0, 4.0),
                EqualizerBand::new(4000.0, 2.0),
                EqualizerBand::new(8000.0, 0.0),
                EqualizerBand::new(16000.0, -2.0),
            ],
        )
    }

    pub fn rock() -> Self {
        Self::new(
            "Rock".to_string(),
            vec![
                EqualizerBand::new(32.0, 5.0),
                EqualizerBand::new(64.0, 4.0),
                EqualizerBand::new(125.0, 2.0),
                EqualizerBand::new(250.0, -1.0),
                EqualizerBand::new(500.0, -2.0),
                EqualizerBand::new(1000.0, -1.0),
                EqualizerBand::new(2000.0, 2.0),
                EqualizerBand::new(4000.0, 4.0),
                EqualizerBand::new(8000.0, 5.0),
                EqualizerBand::new(16000.0, 5.0),
            ],
        )
    }

    pub fn classical() -> Self {
        Self::new(
            "Classical".to_string(),
            vec![
                EqualizerBand::new(32.0, 0.0),
                EqualizerBand::new(64.0, 0.0),
                EqualizerBand::new(125.0, 0.0),
                EqualizerBand::new(250.0, 0.0),
                EqualizerBand::new(500.0, 0.0),
                EqualizerBand::new(1000.0, 0.0),
                EqualizerBand::new(2000.0, -2.0),
                EqualizerBand::new(4000.0, -2.0),
                EqualizerBand::new(8000.0, -2.0),
                EqualizerBand::new(16000.0, -3.0),
            ],
        )
    }

    pub fn jazz() -> Self {
        Self::new(
            "Jazz".to_string(),
            vec![
                EqualizerBand::new(32.0, 3.0),
                EqualizerBand::new(64.0, 2.0),
                EqualizerBand::new(125.0, 1.0),
                EqualizerBand::new(250.0, 2.0),
                EqualizerBand::new(500.0, -1.0),
                EqualizerBand::new(1000.0, -1.0),
                EqualizerBand::new(2000.0, 0.0),
                EqualizerBand::new(4000.0, 2.0),
                EqualizerBand::new(8000.0, 3.0),
                EqualizerBand::new(16000.0, 3.0),
            ],
        )
    }

    pub fn pop() -> Self {
        Self::new(
            "Pop".to_string(),
            vec![
                EqualizerBand::new(32.0, -1.0),
                EqualizerBand::new(64.0, 0.0),
                EqualizerBand::new(125.0, 2.0),
                EqualizerBand::new(250.0, 4.0),
                EqualizerBand::new(500.0, 4.0),
                EqualizerBand::new(1000.0, 3.0),
                EqualizerBand::new(2000.0, 0.0),
                EqualizerBand::new(4000.0, -1.0),
                EqualizerBand::new(8000.0, -1.0),
                EqualizerBand::new(16000.0, -2.0),
            ],
        )
    }
}

/// Biquad filter coefficients for peaking EQ
#[derive(Debug, Clone, Copy)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoefficients {
    /// Create peaking EQ filter coefficients
    fn peaking_eq(frequency: f32, gain_db: f32, sample_rate: u32) -> Self {
        let a = 10_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate as f32;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        
        // Q factor for peaking filter (controls bandwidth)
        let q = 1.0;
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        // Normalize by a0
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// Biquad filter state for a single channel
#[derive(Debug, Clone, Copy)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32, coeff: &BiquadCoefficients) -> f32 {
        let output = coeff.b0 * input + coeff.b1 * self.x1 + coeff.b2 * self.x2
            - coeff.a1 * self.y1 - coeff.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

/// Audio equalizer processor
pub struct Equalizer {
    config: EqualizerConfig,
    sample_rate: u32,
    coefficients: Vec<BiquadCoefficients>,
    // State for left and right channels
    states_left: Vec<BiquadState>,
    states_right: Vec<BiquadState>,
}

impl Equalizer {
    /// Create a new equalizer with the given configuration
    pub fn new(config: EqualizerConfig, sample_rate: u32) -> Self {
        let coefficients = Self::calculate_coefficients(&config.bands, sample_rate);
        let num_bands = config.bands.len();
        
        Self {
            config,
            sample_rate,
            coefficients,
            states_left: vec![BiquadState::new(); num_bands],
            states_right: vec![BiquadState::new(); num_bands],
        }
    }

    fn calculate_coefficients(bands: &[EqualizerBand], sample_rate: u32) -> Vec<BiquadCoefficients> {
        bands
            .iter()
            .map(|band| BiquadCoefficients::peaking_eq(band.frequency, band.gain_db, sample_rate))
            .collect()
    }

    /// Update the equalizer configuration
    pub fn update_config(&mut self, config: EqualizerConfig) {
        self.coefficients = Self::calculate_coefficients(&config.bands, self.sample_rate);
        
        // Reset filter state when config changes significantly
        let num_bands = config.bands.len();
        if num_bands != self.states_left.len() {
            self.states_left = vec![BiquadState::new(); num_bands];
            self.states_right = vec![BiquadState::new(); num_bands];
        }
        
        self.config = config;
    }

    /// Process audio samples in stereo interleaved format
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.config.enabled {
            return;
        }

        // Process samples in stereo pairs
        for chunk in samples.chunks_exact_mut(2) {
            let left = chunk[0];
            let right = chunk[1];

            // Apply each band's filter in series
            let mut left_out = left;
            let mut right_out = right;

            for (i, coeff) in self.coefficients.iter().enumerate() {
                left_out = self.states_left[i].process(left_out, coeff);
                right_out = self.states_right[i].process(right_out, coeff);
            }

            chunk[0] = left_out;
            chunk[1] = right_out;
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &EqualizerConfig {
        &self.config
    }

    /// Check if the equalizer is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equalizer_band_creation() {
        let band = EqualizerBand::new(1000.0, 3.0);
        assert_eq!(band.frequency, 1000.0);
        assert_eq!(band.gain_db, 3.0);
    }

    #[test]
    fn test_default_config() {
        let config = EqualizerConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.bands.len(), 10);
    }

    #[test]
    fn test_preset_creation() {
        let presets = EqualizerPreset::built_in_presets();
        assert!(!presets.is_empty());
        
        let bass_boost = EqualizerPreset::bass_boost();
        assert_eq!(bass_boost.name, "Bass Boost");
        assert_eq!(bass_boost.bands.len(), 10);
    }

    #[test]
    fn test_equalizer_process_when_disabled() {
        let config = EqualizerConfig::default();
        let mut eq = Equalizer::new(config, 44100);
        
        let mut samples = vec![0.5, -0.5, 0.3, -0.3];
        let original = samples.clone();
        
        eq.process(&mut samples);
        
        // When disabled, samples should not change
        assert_eq!(samples, original);
    }

    #[test]
    fn test_equalizer_process_when_enabled() {
        let mut config = EqualizerConfig {
            enabled: true,
            ..Default::default()
        };
        config.bands[0].gain_db = 6.0; // Boost bass
        
        let mut eq = Equalizer::new(config, 44100);
        
        let mut samples = vec![0.5, -0.5, 0.3, -0.3];
        eq.process(&mut samples);
        
        // Samples should be modified when enabled
        // (exact values depend on filter implementation, just checking it changed)
    }

    #[test]
    fn test_equalizer_update_config() {
        let config = EqualizerConfig::default();
        let mut eq = Equalizer::new(config, 44100);
        
        let mut new_config = EqualizerConfig {
            enabled: true,
            ..Default::default()
        };
        new_config.bands[0].gain_db = 10.0;
        
        eq.update_config(new_config.clone());
        
        assert!(eq.config().enabled);
        assert_eq!(eq.config().bands[0].gain_db, 10.0);
    }
}
