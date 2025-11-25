use std::{collections::VecDeque, fs, io, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
}

impl Song {
    pub fn new(title: String, dir: PathBuf) -> Self {
        Self {
            path: dir.join(format!("{title}.ogg")),
            title,
        }
    }

    #[cfg(test)]
    fn mock(title: &str) -> Self {
        Self {
            path: PathBuf::from(title),
            title: title.to_string(),
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
            queue: VecDeque::with_capacity(max_size),
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

    /// Remove some song from the queue by index
    pub fn remove(&mut self, index: usize) {
        let _ = self.queue.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_to_queue() {
        let mut queue = SongQueue::new(10);
        queue.insert(Song::mock("a")).unwrap();
        queue.insert(Song::mock("b")).unwrap();
        queue.insert(Song::mock("c")).unwrap();
        queue.insert(Song::mock("d")).unwrap();

        assert_eq!(
            queue.queue,
            vec![
                Song::mock("d"),
                Song::mock("c"),
                Song::mock("b"),
                Song::mock("a")
            ]
        )
    }

    #[test]
    fn remove_old_songs() {
        let mut queue = SongQueue::new(3);

        for i in 0..=5 {
            let name = i.to_string();
            queue.insert(Song::mock(&name)).unwrap();
        }

        assert_eq!(
            queue.queue,
            vec![Song::mock("5"), Song::mock("4"), Song::mock("3")]
        );
    }
}
