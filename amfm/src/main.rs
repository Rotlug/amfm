use std::{
    error::Error,
    fs,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use antenna::{
    cache::{self, CacheResult},
    playback::{PlaybackManager, PlaybackUpdate},
    stations::{Station, StationList},
};

use arboard::Clipboard;
use clap::Parser;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::{ListState, TableState},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    config::Config,
    song_queue::{Song, SongQueue},
};

mod config;
mod loading_screen;
mod play_screen;
mod radio_info;
mod shortcuts_display;
mod song_queue;
mod stations_table;
mod utils;

pub struct AppModel {
    pub running_state: RunningState,
    pub screen: Screen,

    pub loading_percentage: u64,
    pub loading_result: Option<CacheResult>,

    pub stations: Vec<Station>,
    pub stations_table_state: TableState,
    pub table_virtual_offset: usize,
    pub table_size: u16,

    pub stations_search: Input,
    pub search_toggled: bool,

    pub last_selected_station: usize,

    pub playback: PlaybackManager,
    pub playback_receiver: Receiver<PlaybackUpdate>,

    pub current_station: Option<Station>,

    pub queue: SongQueue,
    pub queue_list_state: ListState,

    pub focus: FocusRegion,

    pub config: Config,

    pub last_update: PlaybackUpdate,

    pub clipboard: Option<Clipboard>,
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

        let mut stations_table_state = TableState::default();
        stations_table_state.select(Some(0));

        Self {
            stations,
            running_state: RunningState::Running,
            screen,
            loading_percentage: 0,
            loading_result,
            playback: mgr,
            playback_receiver: rx,
            current_station: None,
            stations_search: "".into(),
            last_selected_station: 0,
            queue: SongQueue::new(10),
            queue_list_state: ListState::default(),
            stations_table_state,
            focus: FocusRegion::MainArea,
            config: Config::parse(),
            search_toggled: false,
            last_update: PlaybackUpdate::Loading,
            table_size: 0,
            table_virtual_offset: 0,
            clipboard: Clipboard::new().ok(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Screen {
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
    ToggleSearch(bool),
    SearchEvent(Event),
    CopyStationURL,
    StopPlayback,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FocusRegion {
    MainArea,
    RadioInfo,
    Queue,
}

#[derive(PartialEq, Eq, Debug)]
pub enum RunningState {
    Running,
    Done,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut model = AppModel::new();

    let config = Config::parse();

    if let Some(station) = config.station() {
        play_station(&mut model, &station);
    }

    while model.running_state != RunningState::Done {
        // Render
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        while let Some(msg) = current_msg {
            current_msg = update(&mut model, msg)
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
                fs::remove_dir_all(&model.config.temp_song_location)
                    .expect("Could not delete temporary directory");
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
            model.last_update = msg.clone();
            if let PlaybackUpdate::NewSong(tags) = msg {
                model.playback.stop_recording(true);

                let song = Song::new(tags, model.config.temp_song_location.clone());

                if model.config.record && !model.queue.song_exists(&song.tags.title) {
                    model.playback.start_recording(&song.path);

                    model
                        .queue
                        .insert(song)
                        .expect("Error inserting new song to queue");
                }
            }
        }
        Message::StopPlayback => stop(model),
        Message::Navigation(key) => {
            if let Some(new_focus) = handle_navigation(model, key) {
                model.focus = new_focus;
                model
                    .queue_list_state
                    .select(if model.focus == FocusRegion::Queue {
                        Some(0)
                    } else {
                        None
                    });

                model
                    .stations_table_state
                    .select(Some(model.last_selected_station));
            }
        }
        Message::Selection => {
            match model.focus {
                FocusRegion::Queue => {
                    if let Some(index) = model.queue_list_state.selected()
                && index != 0 // First song still being recorded
                && let Some(song) = model.queue.get(index)
                    {
                        // Save song permanently
                        fs::rename(
                            &song.path,
                            model
                                .config
                                .saved_song_location
                                .join(format!("{}.ogg", song.tags.title)),
                        )
                        .expect("Could not save song permanently!");

                        // Remove from queue
                        model.queue.remove(index);
                    }
                }
                FocusRegion::MainArea => {
                    if let Some(index) = model.stations_table_state.selected() {
                        let station = {
                            model
                                .stations
                                .search(model.stations_search.value())
                                .skip(model.table_virtual_offset)
                                .nth(index)
                        };

                        if let Some(station) = station {
                            play_station(model, station.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        Message::ToggleSearch(toggled) => {
            model.search_toggled = toggled;
            model.focus = FocusRegion::MainArea;

            if !toggled {
                model.table_virtual_offset = 0;
                model.last_selected_station = 0;
                model.stations_table_state.select(Some(0));
            }
        }
        Message::SearchEvent(event) => {
            model.table_virtual_offset = 0;
            model.stations_search.handle_event(&event);
        }
        Message::CopyStationURL => {
            if let Some(cb) = &mut model.clipboard
                && let Some(station) = &model.current_station
            {
                let _ = cb.set_text(&station.url);
            }
        }
    }

    None
}

/// Play a station
fn play_station(model: &mut AppModel, station: Station) {
    stop(model);
    model.playback.set_source_uri(&station.url);
    model.current_station = Some(station);
    model.playback.play();
}

/// Stop playback
fn stop(model: &mut AppModel) {
    model.playback.stop_recording(true);
    model.playback.stop();
}

fn handle_navigation(model: &mut AppModel, key: KeyCode) -> Option<FocusRegion> {
    match key {
        KeyCode::Right => match model.focus {
            FocusRegion::MainArea => Some(FocusRegion::Queue),
            _ => None,
        },
        KeyCode::Left => match model.focus {
            FocusRegion::RadioInfo | FocusRegion::Queue => Some(FocusRegion::MainArea),
            _ => None,
        },
        KeyCode::Up => match model.focus {
            FocusRegion::Queue => match model.queue_list_state.selected() {
                Some(0) | None => Some(FocusRegion::RadioInfo),
                _ => {
                    model.queue_list_state.select_previous();
                    None
                }
            },
            FocusRegion::MainArea => {
                let index = model.stations_table_state.selected()?;
                if index == 0 && model.table_virtual_offset > 0 {
                    model.table_virtual_offset -= 1;
                }

                model.stations_table_state.select_previous();
                if let Some(index) = model.stations_table_state.selected() {
                    model.last_selected_station = index;
                }

                None
            }
            _ => None,
        },
        KeyCode::Down => match model.focus {
            FocusRegion::RadioInfo => Some(FocusRegion::Queue),
            FocusRegion::Queue => {
                model.queue_list_state.select_next();
                None
            }
            FocusRegion::MainArea => {
                let next_sel = model.stations_table_state.selected()? + 1;
                if next_sel >= model.table_size as usize {
                    model.table_virtual_offset += 1;
                }

                model.stations_table_state.select_next();
                if let Some(index) = model.stations_table_state.selected() {
                    model.last_selected_station = index;
                }
                None
            }
        },
        _ => None,
    }
}

fn handle_event(model: &AppModel) -> Result<Option<Message>, Box<dyn Error>> {
    if model.screen == Screen::Loading {
        let Some(cr) = &model.loading_result else {
            return Ok(Some(Message::ChangeScreen(Screen::Play)));
        };

        let rx = &cr.rx;

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
        if let KeyCode::Char('/') | KeyCode::Enter | KeyCode::Esc = key.code {
        } else if model.search_toggled {
            return Ok(Some(Message::SearchEvent(Event::Key(key))));
        }

        return Ok(handle_key(model, key));
    }

    Ok(None)
}

fn handle_key(model: &AppModel, key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('y') => Some(Message::CopyStationURL),
        KeyCode::Char('s') => Some(Message::StopPlayback),
        KeyCode::Char('/') => Some(Message::ToggleSearch(!model.search_toggled)),
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            Some(Message::Navigation(key.code))
        }
        KeyCode::Esc => Some(Message::ToggleSearch(false)),
        KeyCode::Enter => {
            if model.search_toggled {
                Some(Message::ToggleSearch(false))
            } else {
                Some(Message::Selection)
            }
        }
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
            model.table_size = if frame.area().height < 4 {
                1
            } else {
                frame.area().height - 4
            };

            frame.render_widget(play_screen::PlayScreen { model }, frame.area());

            if model.search_toggled {
                frame.set_cursor_position((
                    model.stations_search.cursor() as u16,
                    frame.area().height - 2,
                ));
            }
        }
    }
}
