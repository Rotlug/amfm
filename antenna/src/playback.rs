use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use gstreamer::glib::object::ObjectExt;
use gstreamer::glib::{self, clone};
use gstreamer::prelude::{ElementExtManual, GstObjectExt, PadExtManual};
use gstreamer::{MessageView, PadProbeType};
use gstreamer::{
    format::UnsignedIntoSigned,
    glib::object::Cast,
    prelude::{ClockExt, ElementExt, GstBinExt, PadExt},
};

#[derive(Default, Debug)]
struct BufferingState {
    buffering: bool,
    buffering_probe: Option<(gstreamer::Pad, gstreamer::PadProbeId)>,
    is_live: Option<bool>,
}

impl BufferingState {
    fn reset(&mut self) {
        self.buffering = false;
        if let Some((pad, probe_id)) = self.buffering_probe.take() {
            pad.remove_probe(probe_id);
        }
        self.is_live = None;
    }
}

#[derive(Debug)]
pub enum PlaybackUpdate {
    Playing,
    Stopped,
    NewSong(String),
    Error(String),
    Loading,
}

pub struct PlaybackManager {
    pipeline: gstreamer::Pipeline,
    recorderbin: Option<gstreamer::Bin>,
    current_title: Arc<Mutex<String>>,
    sender: Sender<PlaybackUpdate>,

    buffering_state: Arc<Mutex<BufferingState>>,

    // Thread management
    stop_flag: Arc<AtomicBool>,
}

impl PlaybackManager {
    pub fn new(sender: Sender<PlaybackUpdate>) -> Self {
        gstreamer::init().unwrap();

        // create gstreamer pipeline
        let pipeline_description = "uridecodebin name=uridecodebin use-buffering=true buffer-duration=6000000000 ! audioconvert name=audioconvert ! tee name=tee ! queue ! autoaudiosink name=autoaudiosink".to_string();

        let pipeline =
            gstreamer::parse::launch(&pipeline_description).expect("Unable to create pipeline");
        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().unwrap();
        pipeline.set_message_forward(true); // <-- Forwards all messages from child nodes into the parent bin

        let buffering_state = Arc::new(Mutex::new(BufferingState::default()));

        let mut mgr = Self {
            recorderbin: None,
            pipeline,
            sender,
            buffering_state,
            current_title: Arc::new(Mutex::new(String::new())),

            stop_flag: Arc::new(AtomicBool::new(false)),
        };

        mgr.setup_signals();
        mgr
    }

    // Ensures audio pads are linked automatically
    // Also ensures `self` gets updated if any pipeline component gets replaced
    fn setup_signals(&mut self) {
        // dynamically link uridecodebin element with audioconvert element
        let uridecodebin = self.pipeline.by_name("uridecodebin").unwrap();
        let audioconvert = self.pipeline.by_name("audioconvert").unwrap();
        uridecodebin.connect_pad_added(clone!(
            #[weak]
            audioconvert,
            move |_, src_pad| {
                let sink_pad = audioconvert
                    .static_pad("sink")
                    .expect("Failed to get static sink pad from audioconvert");
                if sink_pad.is_linked() {
                    return; // We are already linked. Ignoring.
                }

                let new_pad_caps = src_pad
                    .current_caps()
                    .expect("Failed to get caps of new pad.");
                let new_pad_struct = new_pad_caps
                    .structure(0)
                    .expect("Failed to get first structure of caps.");
                let new_pad_type = new_pad_struct.name();

                if new_pad_type.starts_with("audio/x-raw") {
                    // check if new_pad is audio
                    let _ = src_pad.link(&sink_pad);
                }
            }
        ));

        // listen for new pipeline / bus messages
        let bus = self.pipeline.bus().expect("Unable to get pipeline bus");

        let pipeline_clone = self.pipeline.clone();
        let sender_clone = self.sender.clone();
        let buffering_state_clone = self.buffering_state.clone();
        let current_title_clone = self.current_title.clone();

        let stop_flag = self.stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::SeqCst) {
                if let Some(message) = bus.timed_pop(gstreamer::ClockTime::from_mseconds(100)) {
                    Self::parse_bus_message(
                        pipeline_clone.clone(),
                        &message,
                        &buffering_state_clone,
                        sender_clone.clone(),
                        current_title_clone.clone(),
                    );
                }
            }
        });
    }
    pub fn set_source_uri(&mut self, source: &str) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
        *self.current_title.lock().unwrap() = String::new();

        let uridecodebin = self.pipeline.by_name("uridecodebin").unwrap();
        uridecodebin.set_property("uri", source);
    }

    fn parse_bus_message(
        pipeline: gstreamer::Pipeline,
        message: &gstreamer::Message,
        buffering_state: &Arc<Mutex<BufferingState>>,
        sender: Sender<PlaybackUpdate>,
        current_title: Arc<Mutex<String>>,
    ) {
        match message.view() {
            // Title changes
            MessageView::Tag(tag) => {
                if let Some(t) = tag.tags().get::<gstreamer::tags::Title>() {
                    let new_title = t.get().to_string();

                    // only send message if title really changed.
                    let mut current_title_locked = current_title.lock().unwrap();
                    if *current_title_locked != new_title {
                        current_title_locked.clone_from(&new_title);
                        sender.send(PlaybackUpdate::NewSong(new_title)).unwrap();
                    }
                }
            }
            MessageView::Buffering(buffering) => {
                let percent = buffering.percent();

                // Wait until buffering is complete before start/resume playing
                let mut buffering_state = buffering_state.lock().unwrap();
                if percent < 100 {
                    if !buffering_state.buffering {
                        buffering_state.buffering = true;
                        sender.send(PlaybackUpdate::Loading).unwrap();

                        if buffering_state.is_live == Some(false) {
                            let tee = pipeline.by_name("tee").unwrap();
                            let sinkpad = tee.static_pad("sink").unwrap();
                            let probe_id = sinkpad
                                .add_probe(
                                    gstreamer::PadProbeType::BLOCK
                                        | gstreamer::PadProbeType::BUFFER
                                        | gstreamer::PadProbeType::BUFFER_LIST,
                                    |_pad, _info| gstreamer::PadProbeReturn::Ok,
                                )
                                .unwrap();

                            buffering_state.buffering_probe = Some((sinkpad, probe_id));
                            let _ = pipeline.set_state(gstreamer::State::Paused);
                        }
                    }
                } else if buffering_state.buffering {
                    buffering_state.buffering = false;
                    sender.send(PlaybackUpdate::Playing).unwrap();

                    if buffering_state.is_live == Some(false) {
                        let _ = pipeline.set_state(gstreamer::State::Playing);
                        if let Some((pad, probe_id)) = buffering_state.buffering_probe.take() {
                            pad.remove_probe(probe_id);
                        }
                    }
                }
            }
            MessageView::Element(element) => {
                // Catch the end-of-stream messages from the filesink
                let structure = element.structure().unwrap();
                if structure.name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap();
                    if let MessageView::Eos(_) = &message.view() {
                        // Get recorderbin from message
                        let recorderbin = match message
                            .src()
                            .and_then(|src| src.clone().downcast::<gstreamer::Bin>().ok())
                        {
                            Some(src) => src,
                            None => return,
                        };

                        // And then asynchronously remove it and set its state to Null
                        pipeline.call_async(move |pipeline| {
                            Self::destroy_recorderbin(pipeline.clone(), recorderbin);
                        });
                    }
                }
            }
            // Error
            MessageView::Error(err) => {
                let msg = err.error().to_string();
                let _ = sender.send(PlaybackUpdate::Error(msg));
            }
            _ => (),
        }
    }

    pub fn stop_recording(&mut self, discard_buffered_data: bool) {
        if !self.is_recording() {
            return;
        }

        let recorderbin = self.recorderbin.clone().unwrap();

        // Get the source pad of the tee that is connected to the recorderbin
        let recorderbin_sinkpad = recorderbin
            .static_pad("sink")
            .expect("Failed to get sink pad from recorderbin");

        let tee_srcpad = match recorderbin_sinkpad.peer() {
            Some(peer) => peer,
            None => return,
        };

        // Once the tee source pad is idle and we wouldn't interfere with any data flow,
        // unlink the tee and the recording bin and finalize the recording bin
        // by sending it an end-of-stream event
        //
        // Once the end-of-stream event is handled by the whole recording bin, we get an
        // end-of-stream message from it in the message handler and the shut down the
        // recording bin and remove it from the pipeline
        tee_srcpad.add_probe(
            PadProbeType::IDLE,
            clone!(
                #[weak(rename_to = pipeline)]
                self.pipeline,
                #[upgrade_or_panic]
                move |tee_srcpad, _| {
                    // Get the parent of the tee source pad, i.e. the tee itself
                    let tee = tee_srcpad
                        .parent()
                        .and_then(|parent| parent.downcast::<gstreamer::Element>().ok())
                        .expect("Failed to get tee source pad parent");

                    // Unlink the tee source pad and then release it
                    let _ = tee_srcpad.unlink(&recorderbin_sinkpad);
                    tee.release_request_pad(tee_srcpad);

                    if !discard_buffered_data {
                        // Asynchronously send the end-of-stream event to the sinkpad as this might block for a
                        // while and our closure here might've been called from the main UI thread
                        let recorderbin_sinkpad = recorderbin_sinkpad.clone();
                        recorderbin.call_async(move |_| {
                            recorderbin_sinkpad.send_event(gstreamer::event::Eos::new());
                        });
                    } else {
                        Self::destroy_recorderbin(pipeline, recorderbin.clone());
                    }

                    // Don't block the pad but remove the probe to let everything
                    // continue as normal
                    gstreamer::PadProbeReturn::Remove
                }
            ),
        );
    }

    fn destroy_recorderbin(pipeline: gstreamer::Pipeline, recorderbin: gstreamer::Bin) {
        // Ignore if the bin was not in the pipeline anymore for whatever
        // reason. It's not a problem
        let _ = pipeline.remove(&recorderbin);
    }

    fn calculate_pipeline_offset(pipeline: &gstreamer::Pipeline) -> u64 {
        let clock_time = pipeline
            .clock()
            .expect("Could not get pipeline clock")
            .time();
        let base_time = pipeline
            .base_time()
            .expect("Could not get pipeline base time");

        *clock_time - *base_time
    }

    fn set_state(&mut self, state: gstreamer::State) {
        if state == gstreamer::State::Playing {
            let mut buffering_state = self.buffering_state.lock().unwrap();
            buffering_state.reset();
        }

        if state == gstreamer::State::Null {
            self.sender.send(PlaybackUpdate::Stopped).unwrap();
            *self.current_title.lock().unwrap() = String::new();
        }

        let res = self.pipeline.set_state(state);

        if state > gstreamer::State::Null && res.is_err() {
            self.sender
                .send(PlaybackUpdate::Error("Error!".to_string()))
                .unwrap(); // FIXME
            let _ = self.pipeline.set_state(gstreamer::State::Null);
            return;
        }

        if state >= gstreamer::State::Paused {
            let mut buffering_state = self.buffering_state.lock().unwrap();
            if buffering_state.is_live.is_none() {
                let is_live = res == Ok(gstreamer::StateChangeSuccess::NoPreroll);
                buffering_state.is_live = Some(is_live);
            }
        }
    }

    pub fn play(&mut self) {
        self.set_state(gstreamer::State::Playing);
        self.stop_flag.store(false, Ordering::SeqCst);
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.set_state(gstreamer::State::Null);
    }

    /// Check if recorder exists
    pub fn is_recording(&self) -> bool {
        self.recorderbin.is_some()
    }

    /// Start recording the stream to some path
    pub fn start_recording(&mut self, path: PathBuf) {
        if self.is_recording() {
            return;
        }

        // Create actual recorderbin
        let description =
            "queue name=queue ! vorbisenc ! oggmux  ! filesink name=filesink async=false";
        let recorderbin = gstreamer::parse::bin_from_description(description, true)
            .expect("Unable to create recorderbin");
        recorderbin.set_message_forward(true);

        // We need to set an offset, otherwise the length of the recorded title would be
        // wrong. Get current clock time and calculate offset
        let offset = Self::calculate_pipeline_offset(&self.pipeline);
        let queue_srcpad = recorderbin
            .by_name("queue")
            .unwrap()
            .static_pad("src")
            .unwrap();
        queue_srcpad.set_offset(offset.into_negative().try_into().unwrap_or_default());

        // Set recording path
        let filesink = recorderbin.by_name("filesink").unwrap();
        filesink.set_property("location", path.to_str().unwrap());

        // First try setting the recording bin to playing: if this fails we know this
        // before it potentially interfered with the other part of the pipeline
        recorderbin
            .set_state(gstreamer::State::Playing)
            .expect("Failed to start recording");

        // Add new recorderbin to the pipeline
        self.pipeline
            .add(&recorderbin)
            .expect("Unable to add recorderbin to pipeline");

        // Get our tee element by name, request a new source pad from it and then link
        // that to our recording bin to actually start receiving data
        let tee = self.pipeline.by_name("tee").unwrap();
        let tee_srcpad = tee
            .request_pad_simple("src_%u")
            .expect("Failed to request new pad from tee");
        let sinkpad = recorderbin
            .static_pad("sink")
            .expect("Failed to get sink pad from recorderbin");

        // Link tee srcpad with the sinkpad of the recorderbin
        tee_srcpad
            .link(&sinkpad)
            .expect("Unable to link tee srcpad with recorderbin sinkpad");

        self.recorderbin = Some(recorderbin);
    }
}
