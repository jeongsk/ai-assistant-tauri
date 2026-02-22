//! Multimodal Input Processing
//! 
//! Handles text, image, and mixed input types for AI processing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "png"),
            ImageFormat::Jpeg => write!(f, "jpeg"),
            ImageFormat::Gif => write!(f, "gif"),
            ImageFormat::WebP => write!(f, "webp"),
            ImageFormat::Bmp => write!(f, "bmp"),
        }
    }
}

/// Image data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Input types for multimodal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    /// Plain text input
    Text(String),
    /// Image input with format
    Image { 
        data: Vec<u8>, 
        format: ImageFormat 
    },
    /// Mixed input with text and images
    Mixed { 
        text: String, 
        images: Vec<ImageData> 
    },
}

/// Result of image analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysis {
    /// Generated caption for the image
    pub caption: String,
    /// Detected objects with confidence scores
    pub objects: Vec<ObjectDetection>,
    /// Extracted text (OCR)
    pub extracted_text: Option<String>,
    /// Image tags/categories
    pub tags: Vec<String>,
    /// Confidence score for the analysis
    pub confidence: f32,
}

/// Detected object in image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDetection {
    pub label: String,
    pub confidence: f32,
    pub bounding_box: Option<BoundingBox>,
}

/// Bounding box coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Vision provider trait for abstraction
pub trait VisionProvider: Send + Sync {
    /// Analyze an image and return results
    fn analyze(&self, image_data: &[u8], format: &ImageFormat) -> Result<ImageAnalysis, String>;
    
    /// Extract text from image using OCR
    fn extract_text(&self, image_data: &[u8]) -> Result<String, String>;
    
    /// Generate caption for image
    fn caption(&self, image_data: &[u8]) -> Result<String, String>;
}

/// OCR engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    pub language: String,
    pub dpi: Option<u32>,
}

/// Multimodal processor for handling mixed input
pub struct MultimodalProcessor {
    vision_provider: Option<Box<dyn VisionProvider>>,
    ocr_config: OcrConfig,
}

impl MultimodalProcessor {
    /// Create a new multimodal processor
    pub fn new(vision_provider: Option<Box<dyn VisionProvider>>) -> Self {
        Self {
            vision_provider,
            ocr_config: OcrConfig {
                language: "eng".to_string(),
                dpi: None,
            },
        }
    }
    
    /// Process input and return analysis
    pub fn process(&self, input: &InputType) -> Result<MultimodalResult, String> {
        match input {
            InputType::Text(text) => {
                Ok(MultimodalResult::Text(text.clone()))
            }
            InputType::Image { data, format } => {
                self.process_image(data, format)
            }
            InputType::Mixed { text, images } => {
                self.process_mixed(text, images)
            }
        }
    }
    
    /// Process single image
    fn process_image(&self, data: &[u8], format: &ImageFormat) -> Result<MultimodalResult, String> {
        if let Some(ref provider) = self.vision_provider {
            let analysis = provider.analyze(data, format)?;
            Ok(MultimodalResult::Image(analysis))
        } else {
            Err("No vision provider configured".to_string())
        }
    }
    
    /// Process mixed input
    fn process_mixed(&self, text: &str, images: &[ImageData]) -> Result<MultimodalResult, String> {
        if let Some(ref provider) = self.vision_provider {
            let mut analyses = Vec::new();
            for img in images {
                let analysis = provider.analyze(&img.data, &img.format)?;
                analyses.push(analysis);
            }
            Ok(MultimodalResult::Mixed {
                text: text.to_string(),
                image_analyses: analyses,
            })
        } else {
            // Fallback to text only if no vision provider
            Ok(MultimodalResult::Text(text.to_string()))
        }
    }
    
    /// Extract text from image
    pub fn extract_text(&self, image_data: &[u8]) -> Result<String, String> {
        if let Some(ref provider) = self.vision_provider {
            provider.extract_text(image_data)
        } else {
            Err("No vision provider configured for OCR".to_string())
        }
    }
    
    /// Set OCR configuration
    pub fn set_ocr_config(&mut self, config: OcrConfig) {
        self.ocr_config = config;
    }
}

/// Result of multimodal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultimodalResult {
    /// Text result
    Text(String),
    /// Image analysis result
    Image(ImageAnalysis),
    /// Mixed input result
    Mixed {
        text: String,
        image_analyses: Vec<ImageAnalysis>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_display() {
        assert_eq!(ImageFormat::Png.to_string(), "png");
        assert_eq!(ImageFormat::Jpeg.to_string(), "jpeg");
    }

    #[test]
    fn test_input_type_serialization() {
        let input = InputType::Text("Hello".to_string());
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_multimodal_processor_text() {
        let processor = MultimodalProcessor::new(None);
        let input = InputType::Text("Test input".to_string());
        let result = processor.process(&input).unwrap();
        
        match result {
            MultimodalResult::Text(text) => assert_eq!(text, "Test input"),
            _ => panic!("Expected text result"),
        }
    }

    #[test]
    fn test_image_analysis() {
        let analysis = ImageAnalysis {
            caption: "A cat".to_string(),
            objects: vec![],
            extracted_text: None,
            tags: vec!["animal".to_string(), "pet".to_string()],
            confidence: 0.95,
        };
        
        assert_eq!(analysis.caption, "A cat");
        assert_eq!(analysis.tags.len(), 2);
    }
}
