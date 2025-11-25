use std::{collections::VecDeque, fs, io, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
}

impl From<&str> for Song {
    fn from(value: &str) -> Self {
        Self {
            path: PathBuf::from(format!("/tmp/{value}.ogg")),
            title: value.to_owned(),
        }
    }
}

impl From<String> for Song {
    fn from(value: String) -> Self {
        Self {
            path: PathBuf::from(format!("/tmp/{value}.ogg")),
            title: value.to_owned(),
        }
    }
}

/// Removing old files
#[derive(Debug)]
pub struct SongQueue {
    /// Maximum amount of files to keep at once
    max_size: usize,

    queue: VecDeque<Song>,
}

impl SongQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            queue: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, song: Song) -> Result<(), io::Error> {
        if self.queue.len() == self.max_size
            && let Some(last) = self.queue.pop_back()
            && last.path.exists()
        {
            fs::remove_file(last.path)?;
        }

        self.queue.push_front(song);
        Ok(())
    }

    /// Used to delete all temporary songs at the end of the program
    pub fn discard(&self) {
        for song in &self.queue {
            if song.path.exists() {
                let _ = fs::remove_file(&song.path);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Song> {
        self.queue.iter()
    }

    /// Provides a reference to a song in the queue with the provided
    /// index
    pub fn get(&self, index: usize) -> Option<&Song> {
        self.queue.get(index)
    }

    /// Looks up a song in the queue based on the title
    /// And provides a reference to that song
    pub fn find_by_title(&self, title: &str) -> Option<&Song> {
        self.queue.iter().find(|s| s.title == title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_to_queue() {
        let mut queue = SongQueue::new(10);
        queue.insert(Song::from("a")).unwrap();
        queue.insert(Song::from("b")).unwrap();
        queue.insert(Song::from("c")).unwrap();
        queue.insert(Song::from("d")).unwrap();

        assert_eq!(
            queue.queue,
            vec![
                Song::from("d"),
                Song::from("c"),
                Song::from("b"),
                Song::from("a")
            ]
        )
    }

    #[test]
    fn remove_old_songs() {
        let mut queue = SongQueue::new(3);

        for i in 0..=5 {
            queue.insert(Song::from(i.to_string())).unwrap();
        }

        assert_eq!(
            queue.queue,
            vec![Song::from("5"), Song::from("4"), Song::from("3")]
        );
    }
}
