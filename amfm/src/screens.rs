pub mod loading;
pub mod play;

#[derive(PartialEq, Eq, Debug)]
pub enum Screen {
    Loading,
    Play,
}
