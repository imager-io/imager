// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::collections::LinkedList;
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use ffmpeg_dev::extra::defs;
use ffmpeg_dev::sys::{
    self,
    AVFrame,
    AVDictionary,
    AVCodec,
    AVCodecContext,
    AVStream,
    AVPacket,
    AVFormatContext,
    AVOutputFormat,
    AVCodecParameters,
    AVCodecParserContext,
    AVMediaType,
    AVMediaType_AVMEDIA_TYPE_UNKNOWN as AVMEDIA_TYPE_UNKNOWN,
    AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIA_TYPE_VIDEO,
    AVMediaType_AVMEDIA_TYPE_AUDIO as AVMEDIA_TYPE_AUDIO,
    AVMediaType_AVMEDIA_TYPE_DATA as AVMEDIA_TYPE_DATA,
    AVMediaType_AVMEDIA_TYPE_SUBTITLE as AVMEDIA_TYPE_SUBTITLE,
    AVMediaType_AVMEDIA_TYPE_ATTACHMENT as AVMEDIA_TYPE_ATTACHMENT,
    AVMediaType_AVMEDIA_TYPE_NB as AVMEDIA_TYPE_NB,
    AVFMT_NOFILE,
    AVIO_FLAG_WRITE,
    AVRounding_AV_ROUND_NEAR_INF as AV_ROUND_NEAR_INF,
    AVRounding_AV_ROUND_PASS_MINMAX as AV_ROUND_PASS_MINMAX,
    AVCodecID_AV_CODEC_ID_H264 as AV_CODEC_ID_H264,
    AV_INPUT_BUFFER_PADDING_SIZE,
    AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P,
};

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

pub struct RawYuv420p {
    pub width: u32,
    pub height: u32,
    pub bufsize: i32,
    pub linesize: [i32; 4],
    pub data: [*mut u8; 4],
}

impl Drop for RawYuv420p {
    fn drop(&mut self) {
        unsafe {
            if !self.data[0].is_null() {
                sys::av_free(self.data[0] as *mut c_void);
            }
        };
    }
}


impl RawYuv420p {
    pub fn luma_size(&self) -> u32 {
        self.width * self.height
    }
    pub fn chroma_size(&self) -> u32 {
        self.width * self.height / 4
    }
    pub unsafe fn to_vec(&self) -> Vec<u8> {
        let mut output = Vec::<u8>::new();
        let ptr = self.data[0];
        for i in 0 .. self.bufsize as usize {
            let val = ptr.add(i);
            let val = *val;
            output.push(val);
        }
        output
    }
    pub unsafe fn save(&self, path: &str) {
        println!(
            "ffplay -s {}x{} -pix_fmt yuv420p {}",
            self.width,
            self.height,
            path,
        );
        std::fs::write(path, self.to_vec());
    }
    pub unsafe fn new(width: u32, height: u32) -> Self {
        use sys::{
            AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P
        };
        let pix_fmt: sys::AVPixelFormat = AV_PIX_FMT_YUV420P;
        let mut linesize = [0i32; 4];
        let mut data = [
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ];
        let bufsize = sys::av_image_alloc(
            data.as_mut_ptr(),
            linesize.as_mut_ptr(),
            width as i32,
            height as i32,
            pix_fmt,
            1,
        );
        RawYuv420p {
            width,
            height,
            bufsize,
            linesize,
            data,
        }
    }
    pub unsafe fn fill_from_frame(&mut self, frame: *mut AVFrame) {
        use sys::{
            AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P
        };
        assert!(!frame.is_null());
        assert!((*frame).format == AV_PIX_FMT_YUV420P);
        sys::av_image_copy(
            self.data.as_mut_ptr(),
            self.linesize.as_mut_ptr(),
            (*frame).data.as_mut_ptr() as *mut *const u8,
            (*frame).linesize.as_ptr(),
            (*frame).format,
            (*frame).width,
            (*frame).height,
        );
    }
}


struct Decoder {
    fmt_ctx: *mut sys::AVFormatContext,
    video_dec_ctx: *mut sys::AVCodecContext,
    audio_dec_ctx: *mut sys::AVCodecContext,
    width: i32,
    height: i32,
    pix_fmt: sys::AVPixelFormat,
    video_stream: *mut sys::AVStream,
    audio_stream: *mut sys::AVStream,
    src_filename: CString,
    video_dst_filename: CString,
    audio_dst_filename: CString,

    decoded_video: LinkedList<RawYuv420p>,
    decoded_audio: LinkedList<u8>,

    video_dst_data: [*mut u8; 4],
    video_dst_linesize: [i32; 4],
    video_dst_bufsize: i32,

    video_stream_idx: i32,
    audio_stream_idx: i32,
    frame: *mut AVFrame,
    pkt: AVPacket,
    video_frame_count: u32,
    audio_frame_count: u32,
}

static REFCOUNT: i32 = 0;

impl Decoder {
    pub unsafe fn new(
        source_path: &str,
        output_video_path: &str,
        output_audio_path: &str,
    ) -> Self {
        Decoder {
            fmt_ctx: std::ptr::null_mut(),
            video_dec_ctx: std::ptr::null_mut(),
            audio_dec_ctx: std::ptr::null_mut(),
            width: std::mem::zeroed(),
            height: std::mem::zeroed(),
            pix_fmt: std::mem::zeroed(),
            video_stream: std::ptr::null_mut(),
            audio_stream: std::ptr::null_mut(),

            decoded_audio: Default::default(),
            decoded_video: Default::default(),

            src_filename: c_str(source_path),
            video_dst_filename: c_str(output_video_path),
            audio_dst_filename: c_str(output_audio_path),

            video_dst_data: std::mem::zeroed(),
            video_dst_linesize: std::mem::zeroed(),
            video_dst_bufsize: std::mem::zeroed(),

            video_stream_idx: -1,
            audio_stream_idx: -1,
            frame: std::mem::zeroed(),
            pkt: std::mem::zeroed(),
            video_frame_count: 0,
            audio_frame_count: 0,
        }
    }
}



unsafe fn decode_packet(
    got_frame: &mut i32,
    cached: i32,
    decoder: &mut Decoder,
) -> i32 {
    let mut ret: i32 = 0;
    let mut decoded: i32 = decoder.pkt.size;

    *got_frame = 0;

    if (decoder.pkt.stream_index == decoder.video_stream_idx) {
        /* decode video frame */
        ret = sys::avcodec_decode_video2(
            decoder.video_dec_ctx,
            decoder.frame,
            got_frame,
            &mut decoder.pkt,
        );
        if (ret < 0) {
            // fprintf(stderr, "Error decoding video frame (%s)\n", sys::av_err2str(ret));
            eprintln!("Error decoding video frame: {}", ret);
            return ret;
        }

        if (*got_frame > 0) {

            if (
                (*decoder.frame).width != decoder.width ||
                (*decoder.frame).height != decoder.height ||
                (*decoder.frame).format != decoder.pix_fmt
            ) {
                // To handle this change, one could call av_image_alloc again and
                // decode the following frames into another rawvideo file.
                eprintln!("invalid width/height/pixel-format");
                return -1;
            }

            // println!(
            //     "video_frame n:{} coded_n:{}",
            //     {
            //         decoder.video_frame_count += 1;
            //         decoder.video_frame_count
            //     },
            //     (*decoder.frame).coded_picture_number,
            // );

            // copy decoded frame to destination buffer:
            // this is required since rawvideo expects non aligned data
            sys::av_image_copy(
                decoder.video_dst_data.as_mut_ptr(),
                decoder.video_dst_linesize.as_mut_ptr(),
                (*decoder.frame).data.as_ptr() as *mut *const u8,
                (*decoder.frame).linesize.as_ptr(),
                decoder.pix_fmt,
                decoder.width,
                decoder.height,
            );

            // WRITE TO RAWVIDEO FILE
            // fwrite(video_dst_data[0], 1, video_dst_bufsize, video_dst_file);
            {
                let mut output_picture: RawYuv420p = RawYuv420p::new(
                    (*decoder.frame).width as u32,
                    (*decoder.frame).height as u32,
                );
    
                // COPY DECODED FRAME TO DESTINATION BUFFER;
                // THIS IS REQUIRED SINCE RAWVIDEO EXPECTS NON ALIGNED DATA
                sys::av_image_copy(
                    output_picture.data.as_mut_ptr(),
                    output_picture.linesize.as_mut_ptr(),
                    decoder.video_dst_data.as_mut_ptr() as *mut *const u8,
                    decoder.video_dst_linesize.as_mut_ptr(),
                    AV_PIX_FMT_YUV420P,
                    decoder.width,
                    decoder.height,
                );
                decoder.decoded_video.push_back(output_picture);
            }
        }
    } else if (decoder.pkt.stream_index == decoder.audio_stream_idx) {
        // DECODE AUDIO FRAME
        ret = sys::avcodec_decode_audio4(decoder.audio_dec_ctx, decoder.frame, got_frame, &decoder.pkt);
        if (ret < 0) {
            panic!("Error decoding audio frame");
        }
        
        // Some audio decoders decode only part of the packet, and have to be
        // called again with the remainder of the packet data.
        // Sample: fate-suite/lossless-audio/luckynight-partial.shn
        // Also, some decoders might over-read the packet.
        decoded = ffmpeg_dev::extra::defs::sys_ffmin(ret as usize, decoder.pkt.size as usize) as i32;

        if (*got_frame > 0) {
            let mut unpadded_linesize: i32 = {
                (*decoder.frame).nb_samples * sys::av_get_bytes_per_sample((*decoder.frame).format)
            };
            // println!(
            //     "audio_frame n:{} nb_samples:{} pts:{}",
            //     "(todo)",
            //     "(todo)",
            //     "(todo)",
            //     // audio_frame_count++,
            //     // frame->nb_samples,
            //     // sys::av_ts2timestr(frame->pts, &audio_dec_ctx->time_base)),
            // );

            // Write the raw audio data samples of the first plane. This works
            // fine for packed formats (e.g. AV_SAMPLE_FMT_S16). However,
            // most audio decoders output planar audio, which uses a separate
            // plane of audio samples for each channel (e.g. AV_SAMPLE_FMT_S16P).
            // In other words, this code will write only the first audio channel
            // in these cases.
            // You should use libswresample or libavfilter to convert the frame
            // to packed data.
            // fwrite(frame->extended_data[0], 1, unpadded_linesize, audio_dst_file);
            // unimplemented!();
        }
    }

    // If we use frame reference counting, we own the data and need
    // to de-reference it when we don't use it anymore
    if ((*got_frame > 0) && (REFCOUNT > 0)) {
        sys::av_frame_unref(decoder.frame);
    }

    return decoded;
}

unsafe fn open_codec_context(
    stream_idx: &mut i32,
    dec_ctx: &mut *mut AVCodecContext,
    fmt_ctx: *mut AVFormatContext,
    media_type: sys::AVMediaType,
) -> i32 {
    let mut ret: i32;
    let mut stream_index: i32;
    let mut st: *mut AVStream;
    let mut dec: *mut AVCodec = std::ptr::null_mut();
    let mut opts: *mut AVDictionary = std::ptr::null_mut();

    ret = sys::av_find_best_stream(
        fmt_ctx,
        media_type,
        -1,
        -1,
        std::ptr::null_mut(),
        0,
    );
    if (ret < 0) {
        println!("Could not find %s stream in input file");
        return ret;
    } else {
        stream_index = ret;
        st = *(*fmt_ctx).streams.offset(stream_index as isize);
        assert!(!st.is_null());
        assert!(!(*st).codecpar.is_null());

        // FIND DECODER FOR THE STREAM
        dec = sys::avcodec_find_decoder((*(*st).codecpar).codec_id);
        if (dec.is_null()) {
            unimplemented!("Failed to find codec");
            return unimplemented!();
        }
        

        // ALLOCATE A CODEC CONTEXT FOR THE DECODER
        *dec_ctx = sys::avcodec_alloc_context3(dec);
        if (dec_ctx.is_null()) {
            panic!("Failed to allocate the codec context");
            return unimplemented!();
        }

        // COPY CODEC PARAMETERS FROM INPUT STREAM TO OUTPUT CODEC CONTEXT
        ret = sys::avcodec_parameters_to_context(*dec_ctx, (*st).codecpar);
        if (ret < 0) {
            panic!("Failed to copy codec parameters to decoder context");
            return unimplemented!();
        }

        // INIT THE DECODERS
        ret = sys::avcodec_open2(*dec_ctx, dec, &mut opts);
        if (ret < 0) {
            panic!("Failed to open %s codec");
            return unimplemented!();
        }
        *stream_idx = stream_index;
    }

    return 0;
}

unsafe fn get_format_from_sample_fmt(
    fmt: &mut CString,
    sample_fmt: sys::AVSampleFormat,
) -> i32 {
    use sys::{
        AVSampleFormat,
        AVSampleFormat_AV_SAMPLE_FMT_U8 as AV_SAMPLE_FMT_U8,
        AVSampleFormat_AV_SAMPLE_FMT_S16 as AV_SAMPLE_FMT_S16,
        AVSampleFormat_AV_SAMPLE_FMT_S32 as AV_SAMPLE_FMT_S32,
        AVSampleFormat_AV_SAMPLE_FMT_FLT as AV_SAMPLE_FMT_FLT,
        AVSampleFormat_AV_SAMPLE_FMT_DBL as AV_SAMPLE_FMT_DBL,
    };
    let mut i: i32;

    #[repr(C)]
    struct SampleFmtEntry {
        sample_fmt: sys::AVSampleFormat,
        fmt_be: CString,
        fmt_le: CString,
    }

    let mut sample_fmt_entries: [SampleFmtEntry; 5] = [
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_U8,
            fmt_be: c_str("u8"),
            fmt_le: c_str("u8"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_S16,
            fmt_be: c_str("s16be"),
            fmt_le: c_str("s16le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_S32,
            fmt_be: c_str("s32be"),
            fmt_le: c_str("s32le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_FLT,
            fmt_be: c_str("f32be"),
            fmt_le: c_str("f32le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_DBL,
            fmt_be: c_str("f64be"),
            fmt_le: c_str("f64le"),
        },
    ];
    *fmt = c_str("");
    let mut i = 0;
    while i < sample_fmt_entries.len() {
        i = i + 1;
        let mut entry: &mut SampleFmtEntry = &mut sample_fmt_entries[i];
        if sample_fmt == entry.sample_fmt {
            if sys::AV_HAVE_BIGENDIAN > 0 {
                *fmt = entry.fmt_be.clone();
            } else {
                *fmt = entry.fmt_le.clone();
            }
            return 0;
        }
    }

    println!("sample format is not supported as output format");
    return -1;
}

unsafe fn process(
    src_filename: &str,
    video_dst_filename: &str,
    audio_dst_filename: &str,
) {
    assert!(PathBuf::from(src_filename).exists());
    let mut got_frame: i32 = std::mem::zeroed();
    let mut ret = 0;
    let mut decoder = Decoder::new(
        src_filename,
        video_dst_filename,
        audio_dst_filename,
    );

    std::fs::write(video_dst_filename, &[]);
    std::fs::write(audio_dst_filename, &[]);


    // OPEN INPUT FILE, AND ALLOCATE FORMAT CONTEXT
    if (sys::avformat_open_input(
        &mut decoder.fmt_ctx,
        c_str(src_filename).as_ptr(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ) < 0) {
        panic!("Could not open source file");
    }

    // RETRIEVE STREAM INFORMATION
    if (sys::avformat_find_stream_info(decoder.fmt_ctx, std::ptr::null_mut()) < 0) {
        panic!("Could not find stream information");
    }

    if (open_codec_context(
        &mut decoder.video_stream_idx,
        &mut decoder.video_dec_ctx,
        decoder.fmt_ctx,
        AVMEDIA_TYPE_VIDEO,
    ) >= 0) {
        decoder.video_stream = (*(*decoder.fmt_ctx).streams).offset(decoder.video_stream_idx as isize);
        
        // ALLOCATE IMAGE WHERE THE DECODED IMAGE WILL BE PUT
        decoder.width = (*decoder.video_dec_ctx).width;
        decoder.height = (*decoder.video_dec_ctx).height;
        decoder.pix_fmt = (*decoder.video_dec_ctx).pix_fmt;
        ret = sys::av_image_alloc(
            decoder.video_dst_data.as_mut_ptr(),
            decoder.video_dst_linesize.as_mut_ptr(),
            decoder.width,
            decoder.height,
            decoder.pix_fmt,
            1,
        );
        if (ret < 0) {
            panic!("Could not allocate raw video buffer");
        }
        decoder.video_dst_bufsize = ret;
    }

    if (open_codec_context(
        &mut decoder.audio_stream_idx,
        &mut decoder.audio_dec_ctx,
        decoder.fmt_ctx,
        AVMEDIA_TYPE_AUDIO
    ) >= 0) {
        decoder.audio_stream = (*(*decoder.fmt_ctx).streams).add(decoder.audio_stream_idx as usize);
    }

    // DUMP INPUT INFORMATION TO STDERR
    sys::av_dump_format(
        decoder.fmt_ctx,
        0,
        c_str(src_filename).as_ptr(),
        0,
    );

    if (decoder.audio_stream.is_null() && decoder.video_stream.is_null()) {
        panic!("Could not find audio or video stream in the input, aborting");
    }

    decoder.frame = sys::av_frame_alloc();
    if (decoder.frame.is_null()) {
        panic!("Could not allocate frame");
    }

    // INITIALIZE PACKET - SET DATA TO NULL - LET THE DEMUXER FILL IT
    sys::av_init_packet(&mut decoder.pkt);
    decoder.pkt.data = std::ptr::null_mut();
    decoder.pkt.size = 0;

    if (!decoder.video_stream.is_null()) {
        println!("Demuxing video from file");
    }
    if (!decoder.audio_stream.is_null()) {
        println!("Demuxing audio from file");
    }

    // READ FRAMES FROM THE FILE
    while sys::av_read_frame(decoder.fmt_ctx, &mut decoder.pkt) >= 0 {
        let mut orig_pkt: AVPacket = decoder.pkt.clone();
        {
            ret = decode_packet(&mut got_frame, 0, &mut decoder);
            if (ret < 0) {
                break;
            }
            decoder.pkt.data = decoder.pkt.data.add(ret as usize);
            decoder.pkt.size = decoder.pkt.size - ret;
        }
        while decoder.pkt.size > 0 {
            ret = decode_packet(&mut got_frame, 0, &mut decoder);
            if (ret < 0) {
                break;
            }
            decoder.pkt.data = decoder.pkt.data.add(ret as usize);
            decoder.pkt.size = decoder.pkt.size - ret;
        }
        sys::av_packet_unref(&mut orig_pkt);
    }

    // FLUSH CACHED FRAMES
    decoder.pkt.data = std::ptr::null_mut();
    decoder.pkt.size = 0;
    decode_packet(&mut got_frame, 1, &mut decoder);
    while got_frame > 0 {
        decode_packet(&mut got_frame, 1, &mut decoder);
    }

    println!("Demuxing succeeded");

    // if (video_stream) {
    //     printf("Play the output video file with the command:\n"
    //            "ffplay -f rawvideo -pix_fmt %s -video_size %dx%d %s\n",
    //            av_get_pix_fmt_name(pix_fmt), width, height,
    //            video_dst_filename);
    // }

    // if (audio_stream) {
    //     enum AVSampleFormat sfmt = audio_dec_ctx->sample_fmt;
    //     int n_channels = audio_dec_ctx->channels;
    //     const char *fmt;
    //     if (av_sample_fmt_is_planar(sfmt)) {
    //         const char *packed = av_get_sample_fmt_name(sfmt);
    //         printf("Warning: the sample format the decoder produced is planar "
    //                "(%s). This example will output the first channel only.\n",
    //                packed ? packed : "?");
    //         sfmt = av_get_packed_sample_fmt(sfmt);
    //         n_channels = 1;
    //     }
    //     if ((ret = get_format_from_sample_fmt(&fmt, sfmt)) < 0)
    //         goto end;
    //     printf("Play the output audio file with the command:\n"
    //            "ffplay -f %s -ac %d -ar %d %s\n",
    //            fmt, n_channels, audio_dec_ctx->sample_rate,
    //            audio_dst_filename);
    // }

    // CLEANUP
    sys::avcodec_free_context(&mut decoder.video_dec_ctx);
    sys::avcodec_free_context(&mut decoder.audio_dec_ctx);
    sys::avformat_close_input(&mut decoder.fmt_ctx);
    sys::av_frame_free(&mut decoder.frame);
    sys::av_free(decoder.video_dst_data[0] as *mut c_void);

    // DONE
}


pub fn run() {
    let src_filename = "assets/samples/sintel_trailer-1080p.mp4";
    let video_dst_filename = "assets/output/outvideo";
    let audio_dst_filename = "assets/output/outaudio";
    unsafe {
        process(src_filename, video_dst_filename, audio_dst_filename);
    };
}