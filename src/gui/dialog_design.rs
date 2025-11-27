use iced::{Length, Padding, Color};

pub struct DialogDesign;

impl DialogDesign {
    // Spacing constants - tight but readable
    pub const SPACE_TINY: f32 = 4.0;
    pub const SPACE_SMALL: f32 = 8.0;
    pub const SPACE_MEDIUM: f32 = 12.0;
    pub const SPACE_LARGE: f32 = 16.0;
    
    // Padding constants
    pub const PAD_SMALL: f32 = 8.0;
    pub const PAD_MEDIUM: f32 = 12.0;
    pub const PAD_LARGE: f32 = 16.0;
    
    // Helper functions
    pub fn space_tiny() -> Length {
        Length::Fixed(Self::SPACE_TINY)
    }
    
    pub fn space_small() -> Length {
        Length::Fixed(Self::SPACE_SMALL)
    }
    
    pub fn space_medium() -> Length {
        Length::Fixed(Self::SPACE_MEDIUM)
    }
    
    pub fn space_large() -> Length {
        Length::Fixed(Self::SPACE_LARGE)
    }
    
    pub fn pad_small() -> Padding {
        Padding::new(Self::PAD_SMALL)
    }
    
    pub fn pad_medium() -> Padding {
        Padding::new(Self::PAD_MEDIUM)
    }
    
    pub fn pad_large() -> Padding {
        Padding::new(Self::PAD_LARGE)
    }
    
    // Progress bar height
    pub const PROGRESS_HEIGHT: f32 = 6.0;
    
    // Container border radius
    pub const RADIUS: f32 = 8.0;
}

