use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use antenna::{
    cache::{self, CacheResult},
    playback::{PlaybackManager, PlaybackUpdate},
    stations::Station,
};
use ratatui::{
    crossterm::event::{self, KeyCode},
    widgets::Paragraph,
};

fn mock_loading() -> CacheResult {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut count = 0;

        while count < 100 {
            tx.send(count).unwrap();
            thread::sleep(Duration::from_millis(1));
            count += 1;
        }

        Ok(vec![])
    });

    CacheResult { rx, handle }
}

#[derive(Debug)]
struct AppModel {
    running_state: RunningState,
    screen: Screen,

    loading_percentage: u64,
    loading_result: Option<CacheResult>,

    stations: Vec<Station>,

    playback: PlaybackManager,
    playback_receiver: Receiver<PlaybackUpdate>,
}

impl AppModel {
    fn new() -> Self {
        let data = cache::read_bin_cache();

        let screen;
        let stations;

        let mut loading_result = None;

        if let Ok(data) = data {
            stations = data;
            screen = Screen::Main;
        } else {
            stations = vec![];
            screen = Screen::Loading;
            loading_result = Some(cache::make_cache());
        }

        PlaybackManager::init();

        let (tx, rx) = mpsc::channel();

        let playback = PlaybackManager::new(tx);

        Self {
            running_state: RunningState::Running,
            screen,
            stations,
            loading_percentage: 0,
            loading_result,
            playback,
            playback_receiver: rx,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Screen {
    Loading,
    Main,
}

enum Message {
    Quit,
    LoadingPercentage(u64),
}

#[derive(PartialEq, Eq, Debug)]

enum RunningState {
    Running,
    Done,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut model = AppModel::new();

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
        Message::Quit => model.running_state = RunningState::Done,
        Message::LoadingPercentage(percent) => model.loading_percentage = percent,
    }

    None
}

fn view(model: &mut AppModel, frame: &mut ratatui::Frame) {
    frame.render_widget(
        Paragraph::new(format!("{}", model.loading_percentage.to_string())),
        frame.area(),
    );
}

fn handle_event(model: &AppModel) -> Result<Option<Message>, Box<dyn Error>> {
    if model.screen == Screen::Loading {
        let rx = &model.loading_result.as_ref().unwrap().rx;

        if let Ok(new_perc) = rx.try_recv() {
            return Ok(Some(Message::LoadingPercentage(new_perc)));
        }
    }

    if event::poll(Duration::from_millis(250))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }

    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}
