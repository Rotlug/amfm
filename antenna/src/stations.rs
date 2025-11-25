use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Station {
    pub votes: i32,

    #[serde(rename = "stationuuid")]
    pub id: String,

    pub url: String,
    pub country: String,
    pub name: String,
}

impl Station {
    #[cfg(test)]
    fn mock(name: &str, country: &str) -> Self {
        Self {
            url: "".to_string(),
            id: "".to_string(),
            country: country.to_string(),
            name: name.to_string(),
            votes: 0,
        }
    }
}

/// Trait to implement action on a list of stations (Vec<Station>)
pub trait StationList {
    fn get_all_in_country(&self, country: &str) -> impl Iterator<Item = &Station>;

    fn search(&self, query: &str) -> impl Iterator<Item = &Station>;
}

impl StationList for Vec<Station> {
    fn get_all_in_country(&self, country: &str) -> impl Iterator<Item = &Station> {
        self.iter().filter(move |stat| stat.country == country)
    }

    fn search(&self, query: &str) -> impl Iterator<Item = &Station> {
        let query = query.to_lowercase();
        self.iter().filter(move |station| {
            let name = station.name.to_lowercase();
            name.contains(&query)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_filtering_test() {
        let mock_stations = vec![Station::mock("GlGlZ", "Israel"), Station::mock("BBC", "UK")];

        let filtered: Vec<&Station> = mock_stations.get_all_in_country("Israel").collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].country, "Israel");
    }

    #[test]
    fn search_test() {
        let mock_stations = vec![
            Station::mock("glglz - גלגלצ", "Israel"),
            Station::mock("Radio Bossa", "Brazil"),
        ];
        let search: Vec<&Station> = mock_stations.search("GlGlZ").collect();

        assert_eq!(search.len(), 1);
        assert_eq!(search[0].name, "glglz - גלגלצ")
    }
}
