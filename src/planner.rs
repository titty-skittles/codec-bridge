use crate::media::{MediaFile, MediaStream};

#[derive(Debug)]
pub enum ConversionPlan {
    NoChange,
    ConvertAudioToPcm,
}

pub fn plan_conversion(media: &MediaFile) -> ConversionPlan {
    let has_aac = media.streams.iter().any(|stream| {
        matches!(
            stream,
            MediaStream::Audio(audio) if audio.codec == "aac"
        )
    });

    if has_aac {
        ConversionPlan::ConvertAudioToPcm
    } else {
        ConversionPlan::NoChange
    }
}
