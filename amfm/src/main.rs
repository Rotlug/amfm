use std::{
    error::Error,
    fs,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use antenna::{
    cache::{self, CacheResult},
    playback::{PlaybackManager, PlaybackUpdate},
    stations::Station,
};

use clap::Parser;
use ratatui::{
    crossterm::event::{self, KeyCode},
    widgets::ListState,
};

use crate::{
    config::Config,
    song_queue::{Song, SongQueue},
};

mod config;
mod loading_screen;
mod play_screen;
mod radio_info;
mod song_queue;
mod utils;

#[derive(Debug)]
struct AppModel {
    running_state: RunningState,
    screen: Screen,

    loading_percentage: u64,
    loading_result: Option<CacheResult>,

    stations: Vec<Station>,

    playback: PlaybackManager,
    playback_receiver: Receiver<PlaybackUpdate>,

    current_title: String,
    current_station: Option<Station>,

    queue: SongQueue,
    queue_list_state: ListState,

    focus: FocusRegion,

    config: Config,
}

impl AppModel {
    fn new() -> Self {
        let data = cache::read_bin_cache();

        let screen;
        let stations;

        let mut loading_result = None;

        if let Ok(data) = data {
            stations = data;
            screen = Screen::Play;
        } else {
            stations = vec![];
            screen = Screen::Loading;
            loading_result = Some(cache::make_cache());
        }

        let (tx, rx) = mpsc::channel();

        PlaybackManager::init();
        let mgr = PlaybackManager::new(tx);

        Self {
            running_state: RunningState::Running,
            screen,
            stations,
            loading_percentage: 0,
            loading_result,
            playback: mgr,
            playback_receiver: rx,
            current_title: String::new(),
            current_station: None,
            queue: SongQueue::new(10),
            queue_list_state: ListState::default(),
            focus: FocusRegion::MainArea,
            config: Config::parse(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Screen {
    Loading,
    Play,
}

enum Message {
    Quit,
    ChangeScreen(Screen),
    LoadingPercentage(u64),
    LoadCache,
    PlaybackMsg(PlaybackUpdate),
    Navigation(KeyCode),
    Selection,
    Stop,
}

#[derive(Debug, PartialEq, Eq)]
enum FocusRegion {
    MainArea,
    RadioInfo,
    Queue,
}

#[derive(PartialEq, Eq, Debug)]
enum RunningState {
    Running,
    Done,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut model = AppModel::new();

    let config = Config::parse();
    if let Some(station) = config.station() {
        model.playback.set_source_uri(&station.url);
        model.playback.play();

        model.current_station = Some(station);
    }

    while model.running_state != RunningState::Done {
        // Render
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap())
        }
    }

    ratatui::restore();

    Ok(())
}

fn update(model: &mut AppModel, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = {
                model.playback.stop_recording(true);
                model.queue.discard();
                RunningState::Done
            }
        }
        Message::LoadingPercentage(percent) => model.loading_percentage = percent,
        Message::ChangeScreen(screen) => model.screen = screen,
        Message::LoadCache => {
            model.stations = cache::read_bin_cache().expect("Should have been able to read cache");

            return Some(Message::ChangeScreen(Screen::Play));
        }
        Message::PlaybackMsg(msg) => {
            if let PlaybackUpdate::NewSong(name) = msg {
                model.playback.stop_recording(true);

                model.current_title = name.clone();

                let song = Song::new(name, model.config.temp_song_location.clone());

                model.playback.start_recording(song.path.clone());

                model
                    .queue
                    .insert(song)
                    .expect("Error inserting new song to queue");
            }
        }
        Message::Stop => model.playback.stop(),
        Message::Navigation(key) => {
            if let Some(new_focus) = handle_navigation(model, key) {
                model.focus = new_focus;
            }
        }
        Message::Selection => {
            if let Some(index) = model.queue_list_state.selected()
                && index != 0 // First song still being recorded
                && let Some(song) = model.queue.get(index)
            {
                // Save song permanently
                fs::rename(
                    song.path.clone(),
                    model
                        .config
                        .saved_song_location
                        .join(format!("{}.ogg", song.title)),
                )
                .expect("Could not save song permanently!");

                // Remove from queue
                model.queue.remove(index);
            }
        }
    }

    None
}

fn handle_navigation(model: &mut AppModel, key: KeyCode) -> Option<FocusRegion> {
    match key {
        KeyCode::Right => match model.focus {
            FocusRegion::MainArea => Some(FocusRegion::RadioInfo),
            _ => None,
        },
        KeyCode::Left => match model.focus {
            FocusRegion::RadioInfo | FocusRegion::Queue => {
                model.queue_list_state.select(None);
                Some(FocusRegion::MainArea)
            }
            _ => None,
        },
        KeyCode::Up => match model.focus {
            FocusRegion::Queue => match model.queue_list_state.selected() {
                Some(0) | None => {
                    model.queue_list_state.select(None);
                    Some(FocusRegion::RadioInfo)
                }
                _ => {
                    model.queue_list_state.select_previous();
                    None
                }
            },
            _ => None,
        },
        KeyCode::Down => match model.focus {
            FocusRegion::RadioInfo => {
                model.queue_list_state.select(Some(0));
                Some(FocusRegion::Queue)
            }
            FocusRegion::Queue => {
                model.queue_list_state.select_next();
                None
            }
            _ => None,
        },
        _ => None,
    }
}

fn handle_event(model: &AppModel) -> Result<Option<Message>, Box<dyn Error>> {
    if model.screen == Screen::Loading {
        let rx = &model.loading_result.as_ref().unwrap().rx;

        return match rx.recv() {
            Ok(new_percentage) => Ok(Some(Message::LoadingPercentage(new_percentage))),
            Err(_) => Ok(Some(Message::LoadCache)),
        };
    } else if let Ok(msg) = model.playback_receiver.try_recv() {
        return Ok(Some(Message::PlaybackMsg(msg)));
    }

    if event::poll(Duration::from_millis(250))?
        && let event::Event::Key(key) = event::read()?
        && key.kind == event::KeyEventKind::Press
    {
        return Ok(handle_key(key));
    }

    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('s') => Some(Message::Stop),
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            Some(Message::Navigation(key.code))
        }
        KeyCode::Enter => Some(Message::Selection),
        _ => None,
    }
}

fn view(model: &mut AppModel, frame: &mut ratatui::Frame) {
    match model.screen {
        Screen::Loading => {
            frame.render_widget(
                loading_screen::LoadingScreen {
                    percentage: model.loading_percentage,
                },
                frame.area(),
            );
        }
        Screen::Play => {
            frame.render_widget(
                play_screen::PlayScreen {
                    playback: &model.playback,
                    current_title: &model.current_title,
                    current_station: model.current_station.clone(),
                    queue: &model.queue,
                    queue_list_state: &mut model.queue_list_state,
                    focus: &model.focus,
                },
                frame.area(),
            );
        }
    }
}
