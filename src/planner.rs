use crate::media::{MediaFile, MediaStream};

#[derive(Debug)]
pub enum VideoAction {
    Copy,
    TranscodeDnxhr,
}

#[derive(Debug)]
pub enum AudioAction {
    Copy,
    ConvertToPcm,
}

#[derive(Debug)]
pub struct ConversionPlan {
    pub video: VideoAction,
    pub audio: AudioAction,
}

#[derive(Debug)]
pub enum VideoPreference {
    Preserve,
    ForceDnxhr,
}

#[derive(Debug)]
pub struct PlanningPreferences {
    pub video: VideoPreference,
}

pub fn plan_conversion(
    media: &MediaFile,
    preferences: &PlanningPreferences,
) -> ConversionPlan {
    let has_aac = media.streams.iter().any(|stream| {
        matches!(
            stream,
            MediaStream::Audio(audio) if audio.codec == "aac"
        )
    });

    let audio = if has_aac {
        AudioAction::ConvertToPcm
    } else {
        AudioAction::Copy
    };

    let video = match preferences.video {
        VideoPreference::Preserve => VideoAction::Copy,
        VideoPreference::ForceDnxhr => VideoAction::TranscodeDnxhr,
    };
    
    ConversionPlan {
        video,
        audio,
    }
}

impl VideoAction {
    pub fn description(&self) -> &'static str {
        match self {
            VideoAction::Copy => "Copy",
            VideoAction::TranscodeDnxhr => "Transcode to DNxHR",
        }
    }
}

impl AudioAction {
    pub fn description(&self) -> &'static str {
        match self {
            AudioAction::Copy => "Copy",
            AudioAction::ConvertToPcm => "Convert to PCM",
        }
    }
}
