use super::FetchCallback;
use devand_core::Affinity;
use devand_core::UserAffinity;

use fake::faker::lorem::en::*;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct AffinitiesService {
    callback: FetchCallback,
    rng: StdRng,
}

impl AffinitiesService {
    pub fn new(callback: FetchCallback) -> Self {
        let rng = StdRng::seed_from_u64(42);
        Self { callback, rng }
    }

    pub fn restore(&mut self) {
        self.callback.emit(Ok(fake_affinities(&mut self.rng)))
    }
}

fn fake_affinities(rng: &mut StdRng) -> Vec<UserAffinity> {
    let mut users = Vec::new();
    let n = 6;

    for _ in 0..n {
        let aff_num: f64 = rng.gen_range(0.0, 1.0);
        users.push(UserAffinity::new(
            fake_user(rng),
            Affinity::from_number(aff_num),
        ));
    }

    users
}

fn fake_user(rng: &mut StdRng) -> devand_core::PublicUserProfile {
    use devand_core::*;
    use rand::seq::IteratorRandom;
    use strum::IntoEnumIterator;

    let name: String = Name(EN).fake_with_rng(rng);
    let user_id: i32 = rng.gen_range(1, 1_000_000_000);

    let mut languages = std::collections::BTreeMap::default();

    for lang in Language::iter() {
        if rng.gen_bool(0.2) {
            let level = Level::iter().choose(rng).unwrap();
            let priority = Priority::iter().choose(rng).unwrap();
            languages.insert(lang, LanguagePreference { level, priority });
        }
    }

    let languages = Languages(languages);

    PublicUserProfile {
        id: UserId(user_id),
        username: name
            .to_string()
            .to_lowercase()
            .chars()
            .filter(|x| x.is_alphabetic())
            .collect(),
        visible_name: name.to_string(),
        languages,
    }
}
