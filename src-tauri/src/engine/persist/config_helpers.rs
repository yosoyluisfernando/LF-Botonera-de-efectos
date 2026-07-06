use crate::model::{AppConfig, AudioConfig, PaletaData, ProfileData};

impl AppConfig {
    pub fn active_profile(&self) -> Option<&ProfileData> {
        self.profiles.iter().find(|p| p.id == self.active_profile_id)
    }

    pub fn active_profile_mut(&mut self) -> Option<&mut ProfileData> {
        let id = self.active_profile_id.clone();
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    pub fn active_paleta(&self) -> Option<&PaletaData> {
        let profile = self.active_profile()?;
        profile
            .paletas
            .iter()
            .find(|p| p.id == profile.active_paleta_id)
    }

    pub fn active_paleta_mut(&mut self) -> Option<&mut PaletaData> {
        let profile = self.active_profile_mut()?;
        let id = profile.active_paleta_id.clone();
        profile.paletas.iter_mut().find(|p| p.id == id)
    }

    pub fn active_audio(&self) -> Option<&AudioConfig> {
        self.active_profile().map(|p| &p.audio)
    }
}
