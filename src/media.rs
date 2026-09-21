#[derive(Debug)]
pub struct MediaFile {
    pub streams: Vec<MediaStream>,
    pub duration_seconds: Option<f64>,
}


#[derive(Debug)]
pub enum MediaStream {
    Video(VideoStream),
    Audio(AudioStream),
    Other,
}


#[derive(Debug)]
pub struct VideoStream {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub pixel_format: Option<String>,
}


#[derive(Debug)]
pub struct AudioStream {
    pub codec: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

impl MediaFile {
    pub fn first_video(&self) -> Option<&VideoStream> {
        self.streams.iter().find_map(|stream| {
            match stream {
                MediaStream::Video(video) => Some(video),
                _ => None,
            }
        })
    }

    pub fn first_audio(&self) -> Option<&AudioStream> {
        self.streams.iter().find_map(|stream| {
            match stream {
                MediaStream::Audio(audio) => Some(audio),
                _ => None,
            }
        })
    }
}

