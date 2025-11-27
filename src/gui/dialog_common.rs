use iced::{Length, Padding};

pub struct DialogSpacing;

impl DialogSpacing {
    pub const TINY: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    
    pub fn tiny() -> Length {
        Length::Fixed(Self::TINY)
    }
    
    pub fn small() -> Length {
        Length::Fixed(Self::SMALL)
    }
    
    pub fn medium() -> Length {
        Length::Fixed(Self::MEDIUM)
    }
    
    pub fn large() -> Length {
        Length::Fixed(Self::LARGE)
    }
    
    pub fn padding_small() -> Padding {
        Padding::new(Self::SMALL)
    }
    
    pub fn padding_medium() -> Padding {
        Padding::new(Self::MEDIUM)
    }
    
    pub fn padding_large() -> Padding {
        Padding::new(Self::LARGE)
    }
}


