use anyhow::Result;

/// Trait for implementing slideshow transitions
/// 
/// This trait allows both built-in and custom transitions to be implemented.
/// Each transition defines how to generate FFmpeg filter expressions for
/// transitioning between consecutive slides.
pub trait SlideshowTransition: std::fmt::Debug + Clone {
    /// Get the duration of the transition in seconds
    fn duration(&self) -> f32;
    
    /// Generate FFmpeg filter expression for transitioning between two inputs
    /// 
    /// # Arguments
    /// * `input1` - The first input label (e.g., "[img0]")
    /// * `input2` - The second input label (e.g., "[img1]") 
    /// * `output` - The output label for the transition result (e.g., "[v01]")
    /// * `offset` - Time offset when the transition should start
    /// 
    /// # Returns
    /// FFmpeg filter expression string
    fn to_ffmpeg_filter(&self, input1: &str, input2: &str, output: &str, offset: f32) -> String;
    
    /// Get a human-readable name for the transition
    fn name(&self) -> &str;
    
    /// Validate transition parameters
    fn validate(&self) -> Result<()> {
        if self.duration() <= 0.0 {
            anyhow::bail!("Transition duration must be positive, got: {}", self.duration());
        }
        if self.duration() > 10.0 {
            anyhow::bail!("Transition duration too long (max 10s), got: {}", self.duration());
        }
        Ok(())
    }
}

/// Built-in transition types
#[derive(Debug, Clone, Default)]
pub enum BuiltinTransition {
    /// No transition - simple concatenation
    #[default]
    None,
    /// Crossfade between slides
    Fade { duration: f32 },
    /// Dissolve transition
    Dissolve { duration: f32 },
    /// Slide transition with direction
    Slide { direction: SlideDirection, duration: f32 },
    /// Wipe transition with direction  
    Wipe { direction: WipeDirection, duration: f32 },
}

/// Direction for slide transitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlideDirection {
    Left,
    Right, 
    Up,
    Down,
}

/// Direction for wipe transitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WipeDirection {
    Left,
    Right,
    Up, 
    Down,
    DiagonalTL, // Top-left to bottom-right
    DiagonalTR, // Top-right to bottom-left
}

impl SlideshowTransition for BuiltinTransition {
    fn duration(&self) -> f32 {
        match self {
            BuiltinTransition::None => 0.0,
            BuiltinTransition::Fade { duration } => *duration,
            BuiltinTransition::Dissolve { duration } => *duration,
            BuiltinTransition::Slide { duration, .. } => *duration,
            BuiltinTransition::Wipe { duration, .. } => *duration,
        }
    }
    
    fn to_ffmpeg_filter(&self, input1: &str, input2: &str, output: &str, offset: f32) -> String {
        match self {
            BuiltinTransition::None => {
                // For no transition, we'll handle this differently in the generator
                format!("{}{}concat=n=2:v=1:a=0{}", input1, input2, output)
            },
            BuiltinTransition::Fade { duration } => {
                format!(
                    "{}{}xfade=transition=fade:duration={}:offset={}{}",
                    input1, input2, duration, offset, output
                )
            },
            BuiltinTransition::Dissolve { duration } => {
                format!(
                    "{}{}xfade=transition=dissolve:duration={}:offset={}{}",
                    input1, input2, duration, offset, output
                )
            },
            BuiltinTransition::Slide { direction, duration } => {
                let slide_type = match direction {
                    SlideDirection::Left => "slideleft",
                    SlideDirection::Right => "slideright", 
                    SlideDirection::Up => "slideup",
                    SlideDirection::Down => "slidedown",
                };
                format!(
                    "{}{}xfade=transition={}:duration={}:offset={}{}",
                    input1, input2, slide_type, duration, offset, output
                )
            },
            BuiltinTransition::Wipe { direction, duration } => {
                let wipe_type = match direction {
                    WipeDirection::Left => "wipeleft",
                    WipeDirection::Right => "wiperight",
                    WipeDirection::Up => "wipeup", 
                    WipeDirection::Down => "wipedown",
                    WipeDirection::DiagonalTL => "wipetl",
                    WipeDirection::DiagonalTR => "wipetr",
                };
                format!(
                    "{}{}xfade=transition={}:duration={}:offset={}{}",
                    input1, input2, wipe_type, duration, offset, output
                )
            },
        }
    }
    
    fn name(&self) -> &str {
        match self {
            BuiltinTransition::None => "none",
            BuiltinTransition::Fade { .. } => "fade",
            BuiltinTransition::Dissolve { .. } => "dissolve", 
            BuiltinTransition::Slide { direction, .. } => match direction {
                SlideDirection::Left => "slide-left",
                SlideDirection::Right => "slide-right",
                SlideDirection::Up => "slide-up", 
                SlideDirection::Down => "slide-down",
            },
            BuiltinTransition::Wipe { direction, .. } => match direction {
                WipeDirection::Left => "wipe-left",
                WipeDirection::Right => "wipe-right",
                WipeDirection::Up => "wipe-up",
                WipeDirection::Down => "wipe-down", 
                WipeDirection::DiagonalTL => "wipe-diagonal-tl",
                WipeDirection::DiagonalTR => "wipe-diagonal-tr",
            },
        }
    }
}

impl BuiltinTransition {
    /// Create a fade transition
    pub fn fade(duration: f32) -> Self {
        BuiltinTransition::Fade { duration }
    }
    
    /// Create a dissolve transition  
    pub fn dissolve(duration: f32) -> Self {
        BuiltinTransition::Dissolve { duration }
    }
    
    /// Create a slide transition
    pub fn slide(direction: SlideDirection, duration: f32) -> Self {
        BuiltinTransition::Slide { direction, duration }
    }
    
    /// Create a wipe transition
    pub fn wipe(direction: WipeDirection, duration: f32) -> Self {
        BuiltinTransition::Wipe { direction, duration }
    }
}

/// Parse a transition from a string (for CLI usage)
impl std::str::FromStr for BuiltinTransition {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        let transition_name = parts[0].to_lowercase();
        let duration = if parts.len() > 1 {
            parts[1].parse::<f32>()
                .map_err(|_| anyhow::anyhow!("Invalid duration: {}", parts[1]))?
        } else {
            1.0 // Default duration
        };
        
        match transition_name.as_str() {
            "none" => Ok(BuiltinTransition::None),
            "fade" => Ok(BuiltinTransition::fade(duration)),
            "dissolve" => Ok(BuiltinTransition::dissolve(duration)),
            "slide-left" => Ok(BuiltinTransition::slide(SlideDirection::Left, duration)),
            "slide-right" => Ok(BuiltinTransition::slide(SlideDirection::Right, duration)),
            "slide-up" => Ok(BuiltinTransition::slide(SlideDirection::Up, duration)),
            "slide-down" => Ok(BuiltinTransition::slide(SlideDirection::Down, duration)),
            "wipe-left" => Ok(BuiltinTransition::wipe(WipeDirection::Left, duration)),
            "wipe-right" => Ok(BuiltinTransition::wipe(WipeDirection::Right, duration)),
            "wipe-up" => Ok(BuiltinTransition::wipe(WipeDirection::Up, duration)),
            "wipe-down" => Ok(BuiltinTransition::wipe(WipeDirection::Down, duration)),
            "wipe-diagonal-tl" => Ok(BuiltinTransition::wipe(WipeDirection::DiagonalTL, duration)),
            "wipe-diagonal-tr" => Ok(BuiltinTransition::wipe(WipeDirection::DiagonalTR, duration)),
            _ => anyhow::bail!("Unknown transition type: {}", transition_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_transition_duration() {
        let fade = BuiltinTransition::fade(2.5);
        assert_eq!(fade.duration(), 2.5);
        
        let none = BuiltinTransition::None;
        assert_eq!(none.duration(), 0.0);
    }
    
    #[test]
    fn test_builtin_transition_names() {
        assert_eq!(BuiltinTransition::fade(1.0).name(), "fade");
        assert_eq!(BuiltinTransition::slide(SlideDirection::Left, 1.0).name(), "slide-left");
        assert_eq!(BuiltinTransition::None.name(), "none");
    }
    
    #[test]
    fn test_transition_parsing() {
        let fade: BuiltinTransition = "fade:2.0".parse().unwrap();
        assert_eq!(fade.duration(), 2.0);
        assert_eq!(fade.name(), "fade");
        
        let slide: BuiltinTransition = "slide-left:1.5".parse().unwrap();
        assert_eq!(slide.duration(), 1.5);
        assert_eq!(slide.name(), "slide-left");
        
        let none: BuiltinTransition = "none".parse().unwrap();
        assert_eq!(none.duration(), 0.0);
    }
    
    #[test]
    fn test_validation() {
        let valid = BuiltinTransition::fade(1.0);
        assert!(valid.validate().is_ok());
        
        let invalid = BuiltinTransition::fade(-1.0);
        assert!(invalid.validate().is_err());
        
        let too_long = BuiltinTransition::fade(15.0);
        assert!(too_long.validate().is_err());
    }
}
