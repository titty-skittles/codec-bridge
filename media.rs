#[derive(Debug)]
pub struct MediaFile {
    pub streams: Vec<MediaStream>,
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
